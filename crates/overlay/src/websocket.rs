use crate::SetInfo;
use color_eyre::{eyre::Context, Result};
use leptos::leptos_dom::console_log;
use leptos::leptos_dom::{console_error, console_warn};
use schnose_gsi_client_common::WS_PORT;
use wasm_sockets::{EventClient, Message};

pub fn setup(set_info: SetInfo) -> Result<()> {
	let url = format!("ws://127.0.0.1:{WS_PORT}/overlay");
	let mut ws_client = EventClient::new(&url).context("Failed to create Websocket.")?;

	console_log("Created Websocket client.");

	ws_client.set_on_connection(Some(Box::new(move |ws_client| {
		let message = format!("Established Websocket connection.\n{:#?}", ws_client.status);

		console_warn(&message);
	})));

	ws_client.set_on_message(Some(Box::new(move |_ws_client, message| {
		let msg = format!("Received a message! {message:#?}");
		console_log(&msg);

		if let Message::Text(message) = message {
			match serde_json::from_str(&message) {
				Ok(info) => set_info(Some(info)),
				Err(error) => {
					let message = format!("Failed to serialize message. {error:#?}");

					console_error(&message);
					return;
				}
			};
		} else {
			console_error("Got a message that was not text!");
		}
	})));

	ws_client.set_on_close(Some(Box::new(move |_ev| {
		console_warn("Websocket connection closed!");
	})));

	ws_client.set_on_error(Some(Box::new(move |error| {
		let message = format!("Something happened!\n{}\n{:#?}", error.message(), error.error());
		console_error(&message);
	})));

	Ok(())
}
