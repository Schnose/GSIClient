mod components;
mod util;
mod websocket;

use crate::components::{TimeDisplay, Title};
use leptos::leptos_dom::console_error;
use leptos::*;
use schnose_gsi_client_common::GameInfo;

pub(crate) type GetInfo = ReadSignal<Option<GameInfo>>;
pub(crate) type SetInfo = WriteSignal<Option<GameInfo>>;

fn main() {
	mount_to_body(|cx| {
		view! { cx, <Overlay /> }
	});
}

#[component]
fn Overlay(cx: Scope) -> impl IntoView {
	let (info, set_info) = create_signal::<Option<GameInfo>>(cx, None);

	if let Err(error) = websocket::setup(set_info) {
		let message = format!("Failed to setup Websocket. {error:#?}");
		console_error(&message);
	}

	view! { cx,
		<div>
			<Title info />
		</div>

		<div>
		<TimeDisplay info />
		</div>
	}
}
