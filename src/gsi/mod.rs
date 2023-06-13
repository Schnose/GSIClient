use crate::{Config, GameInfo};
use color_eyre::{
	eyre::{eyre, Context},
	Result,
};
use schnose_gsi::{GSIConfigBuilder, GSIServer, Subscription};
use std::{
	sync::{Arc, Mutex},
	time::Duration,
};
use tokio::sync::broadcast;
use tracing::{debug, error, info, trace};

#[tracing::instrument(skip(sender), "Creating GSI Server")]
pub fn make_server(
	sender: broadcast::Sender<GameInfo>,
	config: Arc<Mutex<Config>>,
) -> Result<schnose_gsi::ServerHandle> {
	let mut config_builder = GSIConfigBuilder::new("schnose-gsi-client");

	config_builder
		.heartbeat(Duration::from_secs(1))
		.subscribe_multiple([
			Subscription::PlayerID,
			Subscription::Map,
		]);

	let gsi_config = config_builder.build();

	let (port, install_dir) = {
		let config = config
			.lock()
			.map_err(|err| eyre!("Failed to acquire mutex guard: {err:?}"))?;

		let port = config.gsi_port;
		let install_dir = match config.cfg_path {
			Some(ref path) if path.exists() && !path.as_os_str().is_empty() => Some(path.clone()),
			_ => None,
		};

		(port, install_dir)
	};

	let mut gsi_server = GSIServer::new(gsi_config, port);

	if let Some(path) = install_dir {
		let err = format!("Failed to install GSI config into `{}`.", path.display());
		gsi_server
			.install_into(path)
			.context(err)?;
	} else {
		gsi_server
			.install()
			.context("Failed to detect cfg directory.")?;
	}

	let sender = Arc::new(sender);
	let gokz_client = gokz_rs::Client::new();
	let prev_info = Arc::new(Mutex::new(None));

	if let Err(error) = sender.send(GameInfo::default()) {
		error!(?error, "Failed to send initial state.");
	}

	gsi_server.add_async_event_listener(move |info| {
		let config = Arc::clone(&config);
		let sender = Arc::clone(&sender);
		// `reqwest::Client` already uses an `Arc`
		let gokz_client = gokz_client.clone();
		let prev_info = Arc::clone(&prev_info);

		Box::pin(async move {
			trace!(event = ?info, "Received new GSI event.");

			{
				let mut prev_info = match prev_info.lock() {
					Ok(guard) => guard,
					Err(error) => {
						error!(?error, "Failed to acquire mutex guard.");
						return;
					}
				};

				if prev_info.as_ref() == Some(&info) {
					debug!("Same event");
					return;
				}

				*prev_info = Some(info.clone());
			}

			let info = match GameInfo::try_from_event(info, &gokz_client).await {
				Ok(info) => info,
				Err(error) => {
					error!(?error, "Failed to parse event.");
					return;
				}
			};

			trace!(?info, "Sending new info");

			if let Err(error) = sender.send(info.clone()) {
				error!(?error, "Failed to send info");
				return;
			}

			let (api_url, api_key) = {
				let config = match config.lock() {
					Ok(guard) => guard,
					Err(error) => {
						error!(?error, "Failed to acquire mutex guard.");
						return;
					}
				};

				(config.api_url.clone(), config.schnose_api_key)
			};

			let Some(api_key) = api_key else {
				trace!("No API key");
				return;
			};

			match gokz_client
				.post(api_url)
				.json(&info)
				.header("x-schnose-api-key", api_key.to_string())
				.send()
				.await
				.map(|res| res.error_for_status())
			{
				Ok(Ok(res)) => {
					trace!(response = ?res, "POSTed to SchnoseAPI");
				}

				Ok(Err(error)) | Err(error) => {
					error!(?error, "Failed to POST to SchnoseAPI");
				}
			};
		})
	});

	info!("Listening for CS:GO events on port {port}");

	gsi_server
		.run()
		.context("Failed to start GSI server.")
}
