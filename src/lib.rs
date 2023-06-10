use gokz_rs::{Mode, SteamID};
use serde::{Deserialize, Serialize};

/// Information about the current state of the game
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameInfo {
	pub player_name: String,
	pub steam_id: SteamID,
	pub map_name: String,
	pub mode: Option<Mode>,
}
