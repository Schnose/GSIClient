use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
	level: Level,
	timestamp: String,
	fields: HashMap<String, JsonValue>,

	#[serde(flatten)]
	rest: JsonValue,
}

impl TryFrom<&[u8]> for Log {
	type Error = String;

	fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
		serde_json::from_slice(buf).map_err(|err| format!("Log data is not valid JSON! {err:?}"))
	}
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "uppercase", untagged)]
enum Level {
	Trace,
	Debug,
	Info,
	Warn,
	Error,
}

impl<'de> Deserialize<'de> for Level {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Ok(match String::deserialize(deserializer)?.as_str() {
			"TRACE" => Self::Trace,
			"DEBUG" => Self::Debug,
			"INFO" => Self::Info,
			"WARN" => Self::Warn,
			"ERROR" => Self::Error,
			level => {
				return Err(serde::de::Error::invalid_value(
					serde::de::Unexpected::Str(level),
					&"expected valid RUST_LOG level",
				))
			}
		})
	}
}
