use axum::{
	body::{boxed, BoxBody},
	extract::{ws::Message, State, WebSocketUpgrade},
	http::Response,
	response::{Html, IntoResponse},
	routing::get,
	Router, Server,
};
use schnose_gsi_client_common::{GameInfo, WS_PORT};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;
use tracing::{error, info, trace};

struct AppState {
	sender: broadcast::Sender<GameInfo>,
}

#[tracing::instrument(skip(sender), "Creating Axum Server")]
pub async fn make_server(sender: broadcast::Sender<GameInfo>) {
	let addr = SocketAddr::from(([127, 0, 0, 1], WS_PORT));
	let state = AppState { sender };

	let router = Router::new()
		.route("/", get(html))
		.route("/health", get(health))
		.route("/overlay", get(overlay))
		.route("/schnose-gsi-client-overlay.js", get(js))
		.route("/schnose-gsi-client-overlay_bg.wasm", get(wasm))
		.with_state(Arc::new(state));

	info!("Starting server...");

	if let Err(error) = Server::bind(&addr)
		.serve(router.into_make_service())
		.await
	{
		error!(?error, "Failed to run Axum server.");
	}
}

#[axum::debug_handler]
async fn health() -> &'static str {
	trace!("Healthcheck!");
	"Healthy"
}

#[axum::debug_handler]
async fn overlay(
	websocket: WebSocketUpgrade,
	State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
	trace!("WebSocket connection!");

	websocket.on_upgrade(|mut websocket| async move {
		let mut receiver = state.sender.subscribe();
		while let Ok(game_info) = receiver.recv().await {
			let json = match serde_json::to_string(&game_info) {
				Ok(json) => json,
				Err(error) => {
					error!(?error, "Failed to serialize game info.");
					continue;
				}
			};

			if let Err(error) = websocket
				.send(Message::Text(json))
				.await
			{
				error!(?error, "Failed to send WebSocket message.");
				return;
			}
		}
	})
}

static OVERLAY_HTML: &str = include_str!("../../../crates/overlay/dist/index.html");
static OVERLAY_JS: &str =
	include_str!("../../../crates/overlay/dist/schnose-gsi-client-overlay.js");

#[axum::debug_handler]
async fn html() -> Html<&'static str> {
	Html(OVERLAY_HTML)
}

#[axum::debug_handler]
async fn js() -> Response<String> {
	Response::builder()
		.header("content-type", "application/javascript;charset=utf-8")
		.body(OVERLAY_JS.to_string())
		.unwrap()
}

#[axum::debug_handler]
async fn wasm() -> Response<BoxBody> {
	Response::builder()
		.header("content-type", "application/wasm")
		.body(boxed(
			include_bytes!("../../../crates/overlay/dist/schnose-gsi-client-overlay_bg.wasm")
				.into_response(),
		))
		.unwrap()
}
