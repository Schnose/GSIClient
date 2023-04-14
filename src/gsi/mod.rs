use {
	crate::config::Config,
	serde::{Deserialize, Serialize},
	std::sync::Arc,
	tokio::sync::Mutex,
	tracing::info,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {}

pub fn run(
	state: Arc<Mutex<Option<State>>>,
	config: Arc<Mutex<Config>>,
) -> schnose_gsi::ServerHandle {
	info!("HELLO GSI");
	todo!()
}
