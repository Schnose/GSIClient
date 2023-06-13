use leptos::leptos_dom::{console_error, console_log};
use leptos::*;
use schnose_gsi_client_common::GameInfo;
use wasm_sockets::{EventClient, Message};

fn main() {
	mount_to_body(|cx| {
		view! { cx, <Overlay /> }
	});
}

#[component]
fn Overlay(cx: Scope) -> impl IntoView {
	let (info, set_info) = create_signal::<Option<GameInfo>>(cx, None);

	setup_websocket(set_info);

	view! { cx,
		<Title info=info/>
	}
}

fn setup_websocket(set_info: WriteSignal<Option<GameInfo>>) {
	let mut client = EventClient::new("ws://127.0.0.1:8869/overlay")
		.expect("Failed to create a Websocket client.");

	console_log("Establishing Websocket connection...");

	client.set_on_error(Some(Box::new(move |error| {
		console_error(&format!("Something happened ({:#?})", error.message()));
	})));

	client.set_on_connection(Some(Box::new(|client| {
		console_log(&format!("Connection established! status={:#?}", client.status));
	})));

	client.set_on_close(Some(Box::new(|_ev| {
		console_log(&format!("Websocket connection closed."));
	})));

	client.set_on_message(Some(Box::new(move |_client, message| {
		console_log(&format!("Got a message! message={message:#?}"));
		if let Message::Text(message) = message {
			if let Ok(info) = serde_json::from_str(&message) {
				set_info(Some(info));
			} else {
				console_error("invalid info!");
			}
		} else {
			console_error("not text!");
		}
	})));
}

#[rustfmt::skip]
macro_rules! time {
	($cx:ident, $info:expr, $runtype_wr:ident, $runtype_pb:ident, $display:expr) => {
		move || {
			let Some($runtype_wr) = $info().and_then(|info| info.$runtype_wr) else {
				return view! { $cx,
					<span class={$display}> {$display} </span> ": no WR"
				};
			};

			let text = format!(": {} by {}", format_time($runtype_wr.time), $runtype_wr.player_name);
			let by = if let Some($runtype_pb) = $info().and_then(|info| info.$runtype_pb) {
				let diff = $runtype_pb.time - $runtype_wr.time;
				if diff > 0.0 {
					let text = format!(" (+{})", format_time(diff));
					view! { $cx,
						<span class="time_diff"> {text} </span>
					}
				} else {
					view! { $cx,
						<span class="wr_by_me"> " (WR by me)" </span>
					}
				}
			} else {
				view!{ $cx, <span></span> }
			};

			view! { $cx,
				<span class={$display}> {$display} </span> {text} {by}
			}
		}
	};
}

#[component]
fn Title(cx: Scope, info: ReadSignal<Option<GameInfo>>) -> impl IntoView {
	let mode = move || {
		info().map(|ref info| {
			macro_rules! fmt {
				($cx:ident, $item:expr) => {
					view! { $cx,
						<span class="mode_bracket"> "[" </span>
							<span class="mode_name"> {$item} </span>
						<span class="mode_bracket"> "] " </span>
					}
				};
			}

			info.mode
				.map_or_else(|| fmt!(cx, "???"), |mode| fmt!(cx, mode.short()))
		})
	};

	let map_name = move || {
		macro_rules! fmt {
			($cx:ident, $item:expr) => {
				view! { $cx,
					<span class="map_name"> {$item} </span>
				}
			};
		}

		info().map_or_else(|| fmt!(cx, "unknown map"), |ref info| fmt!(cx, info.map_name.clone()))
	};

	let tp = time!(cx, info, tp_wr, tp_pb, "TP");
	let pro = time!(cx, info, pro_wr, pro_pb, "PRO");

	view! { cx,
		<h1>
			{mode} {map_name}
		</h1>

		<h3>
			{tp}
		</h3>

		<h3>
			{pro}
		</h3>
	}
}

pub fn format_time(seconds: f64) -> String {
	let hours = (seconds / 3600.0) as u8;
	let minutes = ((seconds % 3600.0) / 60.0) as u8;
	let seconds = seconds % 60.0;

	let mut formatted = format!("{minutes:02}:{seconds:06.3}");

	if hours > 0 {
		formatted = format!("{hours:02}:{formatted}");
	}

	formatted
}
