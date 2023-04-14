use {
	crate::gsi::State,
	axum::{
		extract::{Query, State as StateExtractor},
		http::StatusCode,
		response::{Html, IntoResponse},
		routing::get,
		Json, Router, Server,
	},
	gokz_rs::{
		global_api::{self, Record},
		MapIdentifier, Mode, SteamID,
	},
	serde::{Deserialize, Serialize},
	std::{net::SocketAddr, path::PathBuf, sync::Arc},
	tokio::sync::Mutex,
};

pub const PORT: u16 = 9999;

#[derive(Debug, Clone)]
pub struct StateWrapper {
	state: Arc<Mutex<Option<State>>>,
	gokz_client: gokz_rs::Client,
}

pub async fn run(state: Arc<Mutex<Option<State>>>) {
	let state = StateWrapper {
		state,
		gokz_client: gokz_rs::Client::new(),
	};

	let addr = SocketAddr::from(([127, 0, 0, 1], PORT));

	let router = Router::new()
		.route("/", get(overlay))
		.route("/gsi", get(get_state))
		.route("/wrs", get(get_wrs))
		.route("/pbs", get(get_pbs))
		.with_state(state);

	Server::bind(&addr)
		.serve(router.into_make_service())
		.await
		.expect("Failed to run Axum server.")
}

#[derive(Debug, Clone, Serialize)]
struct Response {
	json: Option<State>,
}

impl IntoResponse for Response {
	fn into_response(self) -> axum::response::Response {
		match self.json {
			None => (StatusCode::NO_CONTENT, ()).into_response(),
			Some(json) => (StatusCode::OK, Json(json)).into_response(),
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
struct GlobalAPIParams {
	pub steam_id: SteamID,
	pub map_identifier: MapIdentifier,
	pub mode: Mode,
}

async fn get_state(
	StateExtractor(StateWrapper { state, .. }): StateExtractor<StateWrapper>,
) -> Response {
	Response { json: state.lock().await.clone() }
}

type Records = (Option<Record>, Option<Record>);

async fn get_wrs(
	Query(GlobalAPIParams { map_identifier, mode, .. }): Query<GlobalAPIParams>,
	StateExtractor(StateWrapper { gokz_client, .. }): StateExtractor<StateWrapper>,
) -> Json<Records> {
	let tp_wr = global_api::get_wr(map_identifier.clone(), mode, true, 0, &gokz_client)
		.await
		.ok();

	let pro_wr = global_api::get_wr(map_identifier, mode, false, 0, &gokz_client)
		.await
		.ok();

	Json((tp_wr, pro_wr))
}

async fn get_pbs(
	Query(GlobalAPIParams { steam_id, map_identifier, mode }): Query<GlobalAPIParams>,
	StateExtractor(StateWrapper { gokz_client, .. }): StateExtractor<StateWrapper>,
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
