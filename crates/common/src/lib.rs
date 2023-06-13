use gokz_rs::{Mode, SteamID, Tier};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameInfo {
	pub player_name: String,
	pub steam_id: Option<SteamID>,
	pub map_name: String,
	pub map_tier: Option<Tier>,
	pub mode: Option<Mode>,
	pub tp_wr: Option<Record>,
	pub tp_pb: Option<Record>,
	pub pro_wr: Option<Record>,
	pub pro_pb: Option<Record>,
}

impl Default for GameInfo {
	fn default() -> Self {
		Self {
			player_name: String::from("unknown"),
			steam_id: None,
			map_name: String::from("unknown map"),
			map_tier: None,
			mode: None,
			tp_wr: None,
			tp_pb: None,
			pro_wr: None,
			pro_pb: None,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
	pub time: f64,
	pub player_name: String,
	pub steam_id: SteamID,
}

impl From<gokz_rs::global_api::Record> for Record {
	fn from(record: gokz_rs::global_api::Record) -> Self {
		Self {
			time: record.time,
			player_name: record.player_name,
			steam_id: record.steam_id,
		}
	}
}
