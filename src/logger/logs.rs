use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {}

impl TryFrom<&[u8]> for Log {
	type Error = &'static str;

	fn try_from(_buf: &[u8]) -> Result<Self, Self::Error> {
		Ok(Self {})
	}
}
