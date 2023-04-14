use {crate::gsi::State, std::sync::Arc, tokio::sync::Mutex, tracing::info};

pub async fn run(state: Arc<Mutex<Option<State>>>) {
	info!("HELLO AXUM");
}
