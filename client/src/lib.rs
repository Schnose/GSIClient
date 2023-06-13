mod config;
pub use config::Config;

mod gsi;
pub use gsi::make_server;

use color_eyre::Result;
use gokz_rs::{global_api, MapIdentifier, Mode, SteamID, Tier};
use schnose_gsi_client_common::GameInfo;

const VALID_MAP_PREFIXES: [&str; 6] = [
	"bkz_", "kz_", "kzpro_", "skz_", "vnl_", "xc_",
];

fn is_valid_map_name(map_name: &str) -> bool {
	VALID_MAP_PREFIXES
		.iter()
		.any(|prefix| map_name.starts_with(prefix))
}
pub async fn info_from_event(
	event: schnose_gsi::Event,
	gokz_client: &gokz_rs::Client,
) -> Result<GameInfo> {
	let mut player_name: Option<String> = None;
	let mut steam_id: Option<SteamID> = None;
	let mut mode: Option<Mode> = None;

	if let Some(player) = event.player {
		player_name = Some(player.name);
		steam_id = SteamID::from_id64(player.steam_id.as_id64()).ok();
		mode = player.clan.and_then(|clan| {
			if clan.contains(' ') {
				clan.split_once(' ')
					.and_then(|(mode, _rank)| mode.replace('[', "").parse().ok())
			} else {
				clan.replace(['[', ']'], "")
					.parse()
					.ok()
			}
		});
	}

	let player_name = player_name.unwrap_or_else(|| String::from("unknown player"));

	let mut map_name: Option<String> = None;
	let mut map_tier: Option<Tier> = None;

	if let Some(map) = event.map {
		'scope: {
			let name = map
				.name
				.rsplit_once('/')
				.map(|(_, map_name)| map_name.to_owned())
				.unwrap_or(map.name);

			if !is_valid_map_name(&name) {
				break 'scope;
			}

			let Ok(map) = global_api::get_map(&MapIdentifier::Name(name), gokz_client).await else {
				break 'scope;
			};

			map_name = Some(map.name);
			map_tier = Some(map.difficulty);
		}
	}

	let map_name = map_name.unwrap_or_else(|| String::from("unknown map"));

	let (tp_wr, tp_pb, pro_wr, pro_pb) = 'scope: {
		let Some(mode) = mode else {
			break 'scope Default::default();
		};

		let tp_wr = global_api::get_wr(map_name.clone().into(), mode, true, 0, gokz_client);
		let tp_pb = global_api::get_pb(
			steam_id
				.map(Into::into)
				.unwrap_or_else(|| player_name.clone().into()),
			map_name.clone().into(),
			mode,
			true,
			0,
			gokz_client,
		);
		let pro_wr = global_api::get_wr(map_name.clone().into(), mode, false, 0, gokz_client);
		let pro_pb = global_api::get_pb(
			steam_id
				.map(Into::into)
				.unwrap_or_else(|| player_name.clone().into()),
			map_name.clone().into(),
			mode,
			false,
			0,
			gokz_client,
		);

		let (tp_wr, tp_pb, pro_wr, pro_pb) =
			futures::future::join4(tp_wr, tp_pb, pro_wr, pro_pb).await;

		(
			tp_wr.map(Into::into).ok(),
			tp_pb.map(Into::into).ok(),
			pro_wr.map(Into::into).ok(),
			pro_pb.map(Into::into).ok(),
		)
	};

	Ok(GameInfo {
		player_name,
		steam_id,
		map_name,
		map_tier,
		mode,
		tp_wr,
		tp_pb,
		pro_wr,
		pro_pb,
	})
}
