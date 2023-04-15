use {
	crate::gsi::State,
	axum::{
		extract::{
			ws::{Message, WebSocketUpgrade},
			Query, State as StateExtractor,
		},
		response::{Html, IntoResponse},
		routing::get,
		Json, Router, Server,
	},
	gokz_rs::{
		global_api::{self, Record},
		MapIdentifier, Mode, SteamID,
	},
	serde::Deserialize,
	std::{net::SocketAddr, path::PathBuf, sync::Arc},
	tokio::sync::broadcast::Receiver,
	tracing::error,
};

pub const PORT: u16 = 9999;

#[derive(Debug, Clone)]
pub struct StateReceiver {
	receiver: Arc<Receiver<State>>,
	gokz_client: Arc<gokz_rs::Client>,
}

pub async fn run(receiver: Receiver<State>) {
	let state_receiver = StateReceiver {
		receiver: Arc::new(receiver),
		gokz_client: Arc::new(gokz_rs::Client::new()),
	};

	let addr = SocketAddr::from(([127, 0, 0, 1], PORT));

	let router = Router::new()
		.route("/", get(overlay))
		.route("/gsi", get(websocket))
		.route("/wrs", get(wrs))
		.route("/pbs", get(pbs))
		.with_state(state_receiver);

	Server::bind(&addr)
		.serve(router.into_make_service())
		.await
		.expect("Failed to run Axum server.")
}

async fn websocket(
	ws: WebSocketUpgrade,
	StateExtractor(StateReceiver { receiver, .. }): StateExtractor<StateReceiver>,
) -> impl IntoResponse {
	ws.on_upgrade(|mut ws| async move {
		let mut receiver = receiver.resubscribe();
		while let Ok(state) = receiver.recv().await {
			let json = match serde_json::to_string(&state) {
				Ok(json) => json,
				Err(why) => {
					error!("Failed to serialize state: {why:?}");
					continue;
				}
			};

			if let Err(why) = ws.send(Message::Text(json)).await {
				error!("Failed to send state: {why:?}")
			}
		}
	})
}

#[derive(Debug, Clone, Deserialize)]
struct GlobalAPIParams {
	pub steam_id: SteamID,
	pub map_identifier: MapIdentifier,
	pub mode: Mode,
}

type Records = (Option<Record>, Option<Record>);

async fn wrs(
	Query(GlobalAPIParams { map_identifier, mode, .. }): Query<GlobalAPIParams>,
	StateExtractor(StateReceiver { gokz_client, .. }): StateExtractor<StateReceiver>,
) -> Json<Records> {
	let tp_wr = global_api::get_wr(map_identifier.clone(), mode, true, 0, &gokz_client)
		.await
		.ok();

	let pro_wr = global_api::get_wr(map_identifier, mode, false, 0, &gokz_client)
		.await
		.ok();

	Json((tp_wr, pro_wr))
}

async fn pbs(
	Query(GlobalAPIParams { steam_id, map_identifier, mode }): Query<GlobalAPIParams>,
	StateExtractor(StateReceiver { gokz_client, .. }): StateExtractor<StateReceiver>,
) -> Json<Records> {
	let tp_pb =
		global_api::get_pb(steam_id.into(), map_identifier.clone(), mode, true, 0, &gokz_client)
			.await
			.ok();

	let pro_pb = global_api::get_pb(steam_id.into(), map_identifier, mode, false, 0, &gokz_client)
		.await
		.ok();

	Json((tp_pb, pro_pb))
}

async fn overlay() -> Html<String> {
	Html(if let Ok(reload_path) = std::env::var("SCHNOSE_GSI_OVERLAY_HOT_RELOAD") {
		let path = PathBuf::from(reload_path);
		tokio::fs::read_to_string(path)
			.await
			.expect("Failed to read in HTML.")
	} else {
		let html = include_str!("../../assets/overlay/index.html");
		let css = include_str!("../../assets/overlay/index.css");
		let js = include_str!("../../assets/overlay/index.js")
			.replace("__REPLACE_PORT__", &PORT.to_string());

		html.replace("__REPLACE_CSS__", css)
			.replace("__REPLACE_JS__", &js)
	})
}
