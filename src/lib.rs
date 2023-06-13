mod config;
pub use config::Config;

mod gsi;
pub use gsi::make_server;

use color_eyre::{eyre::Context, Result};
use gokz_rs::{global_api, MapIdentifier, Mode, SteamID, Tier};
use serde::{Deserialize, Serialize};

/// Information about the current state of the game
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameInfo {
	pub player_name: String,
	pub steam_id: Option<SteamID>,
	pub map_name: String,
	pub map_tier: Option<Tier>,
	pub mode: Option<Mode>,
}

impl Default for GameInfo {
	fn default() -> Self {
		Self {
			player_name: String::from("unknown"),
			steam_id: None,
			map_name: String::from("unknown map"),
			map_tier: None,
			mode: None,
		}
	}
}

impl GameInfo {
	pub async fn try_from_event(
		event: schnose_gsi::Event,
		gokz_client: &gokz_rs::Client,
	) -> Result<Self> {
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

		let mut map_name: Option<String> = None;
		let mut map_tier: Option<Tier> = None;

		if let Some(map) = event.map {
			'scope: {
				let name = map
					.name
					.rsplit_once('/')
					.map(|(_, map_name)| map_name.to_owned())
					.unwrap_or(map.name);

				if !Self::is_valid_map_name(&name) {
					break 'scope;
				}

				let map = global_api::get_map(&MapIdentifier::Name(name), gokz_client)
					.await
					.context("Failed to fetch map from GlobalAPI.")?;

				map_name = Some(map.name);
				map_tier = Some(map.difficulty);
			}
		}

		Ok(Self {
			player_name: player_name.unwrap_or_else(|| String::from("unknown player")),
			steam_id,
			map_name: map_name.unwrap_or_else(|| String::from("unknown map")),
			map_tier,
			mode,
		})
	}

	const VALID_MAP_PREFIXES: [&str; 6] = [
		"bkz_", "kz_", "kzpro_", "skz_", "vnl_", "xc_",
	];

	fn is_valid_map_name(map_name: &str) -> bool {
		Self::VALID_MAP_PREFIXES
			.iter()
			.any(|prefix| map_name.starts_with(prefix))
	}
}
