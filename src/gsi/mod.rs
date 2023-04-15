use {
	crate::config::Config,
	color_eyre::{eyre::Context, Result},
	gokz_rs::{global_api, MapIdentifier, Mode, SteamID, Tier},
	schnose_gsi::{GSIConfigBuilder, GSIServer, Subscription},
	serde::{Deserialize, Serialize},
	std::{sync::Arc, time::Duration},
	tokio::sync::{broadcast::Sender, Mutex},
	tracing::{debug, error, info, trace, warn},
	uuid::Uuid,
};

pub fn run(
	// state: Arc<Mutex<Option<State>>>,
	state_sender: Sender<State>,
	config: Arc<Mutex<Config>>,
) -> schnose_gsi::ServerHandle {
	let mut config_builder = GSIConfigBuilder::new("schnose-gsi-client");

	config_builder
		.heartbeat(Duration::from_secs(1))
		.subscribe_multiple([
			Subscription::Map,
			Subscription::PlayerID,
		]);

	let gsi_config = config_builder.build();

	let (port, detect_install_dir) = tokio::task::block_in_place(|| {
		let config = config.blocking_lock();
		let is_fake = match &config.csgo_cfg_path {
			None => true,
			Some(path) => !path.exists(),
		};
		let is_cwd = match &config.csgo_cfg_path {
			None => true,
			Some(path) => path.as_os_str().is_empty(),
		};

		(config.gsi_port, is_fake || is_cwd)
	});

	let mut gsi_server = GSIServer::new(gsi_config, port);

	if detect_install_dir {
		gsi_server.install().unwrap();
	} else {
		gsi_server
			.install_into(tokio::task::block_in_place(|| {
				config
					.blocking_lock()
					.csgo_cfg_path
					.clone()
					.expect("Config directory may not be empty")
			}))
			.unwrap();
	}

	let state_sender = Arc::new(state_sender);
	let gokz_client = Arc::new(gokz_rs::Client::new());
	let prev_event = Arc::new(Mutex::new(None));

	// Send initial payload
	if let Err(why) = state_sender.send(State::default()) {
		error!("Failed to send new state: {why:?}");
	}

	gsi_server.add_async_event_listener(move |event| {
		let gokz_client = Arc::clone(&gokz_client);
		let state_sender = Arc::clone(&state_sender);
		let config = Arc::clone(&config);
		let prev_event = Arc::clone(&prev_event);

		Box::pin(async move {
			trace!("New GSI Event.");
			debug!("{event:#?}");

			// Check if the new event is the same as the previous one.
			// There is no need to proceed and re-fetch information from the GlobalAPI if nothing
			// changed.
			{
				let mut prev_event = prev_event.lock().await;
				if (*prev_event).as_ref() == Some(&event) {
					warn!("SAME EVENT");
					return;
				}
				*prev_event = Some(event.clone());
			}

			let new_state = match State::from_event(event, &gokz_client).await {
				Ok(state) => state,
				Err(why) => return error!("Failed to parse event: {why:#?}"),
			};

			info!("Sending state: {new_state:?}");

			if let Err(why) = state_sender.send(new_state.clone()) {
				return error!("Failed to send new state: {why:?}");
			}

			let (api_url, schnose_api_key) = {
				let config = config.lock().await;
				(config.api_url.clone(), config.schnose_api_key)
			};

			let Some(schnose_api_key) = schnose_api_key else {
				return;
			};

			if let Err(why) =
				notify_twitch_bot(new_state, &api_url, schnose_api_key, &gokz_client).await
			{
				error!("Failed to notify Twitch Bot with new state: {why:#?}");
			}
		})
	});

	info!("Listening for CS:GO events on port {port}.");

	gsi_server
		.run()
		.expect("Failed to run GSI Server.")
}

async fn notify_twitch_bot(
	state: State,
	api_url: &str,
	api_key: Uuid,
	gokz_client: &gokz_rs::Client,
) -> Result<()> {
	match gokz_client
		.post(api_url)
		.json(&state)
		.header("x-schnose-api-key", api_key.to_string())
		.send()
		.await
		.map(|res| res.error_for_status())
	{
		Ok(Ok(res)) => {
			trace!("Notified Twitch Bot!");
			debug!("{res:#?}");
			Ok(())
		}
		Ok(Err(why)) | Err(why) => {
			error!("Failed to notify Twitch Bot.");
			error!("{why:#?}");
			Err(why.into())
		}
	}
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {
	pub player_name: Option<String>,
	pub steam_id: Option<SteamID>,
	pub map_name: Option<String>,
	pub map_tier: Option<Tier>,
	pub mode: Option<Mode>,
}

impl State {
	const VALID_MAP_PREFIXES: [&str; 6] = [
		"bkz_", "kz_", "kzpro_", "skz_", "vnl_", "xc_",
	];

	pub async fn from_event(
		event: schnose_gsi::Event,
		gokz_client: &gokz_rs::Client,
	) -> Result<Self> {
		let (player_name, steam_id, mode) = event
			.player
			.map(|player| {
				let name = player.name.clone();
				let steam_id = player.steam_id;
				let mode = player
					.clan
					.and_then(|clan| match clan.contains(' ') {
						true => clan
							.split_once(' ')
							.and_then(|(mode, _rank)| {
								mode.replace('[', "")
									.parse::<Mode>()
									.ok()
							}),
						false => clan
							.replace(['[', ']'], "")
							.parse::<Mode>()
							.ok(),
					});

				(Some(name), Some(steam_id), mode)
			})
			.unwrap_or_default();

		let (map_name, map_tier) = match event
			.map
			.map(|map| match map.name.contains('/') {
				true => map
					.name
					.rsplit_once('/')
					.map(|(_, map_name)| map_name.to_owned())
					.unwrap(),
				false => map.name.clone(),
			}) {
			None => (String::from("unknown map"), None),
			Some(map_name) if !Self::is_valid_map_name(&map_name) => {
				(String::from("unknown map"), None)
			}
			Some(map_name) => global_api::get_map(&MapIdentifier::Name(map_name), gokz_client)
				.await
				.map(|map| (map.name, Some(map.difficulty)))
				.context("Failed to fetch map from GlobalAPI.")?,
		};

		Ok(Self {
			player_name,
			steam_id,
			map_name: Some(map_name),
			map_tier,
			mode,
		})
	}

	fn is_valid_map_name(map_name: &str) -> bool {
		Self::VALID_MAP_PREFIXES
			.iter()
			.any(|prefix| map_name.starts_with(prefix))
	}
}
