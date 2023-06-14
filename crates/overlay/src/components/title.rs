#![allow(unused_braces)]

use crate::GetInfo;
use leptos::{view, IntoView, Scope};

#[leptos::component]
pub fn Title(cx: Scope, info: GetInfo) -> impl IntoView {
	view! { cx,
		<Mode info /> " " <MapName info />
	}
}

#[leptos::component]
fn Mode(cx: Scope, info: GetInfo) -> impl IntoView {
	let mode_text = move || 'scope: {
		if let Some(ref info) = info() {
			if let Some(mode) = info.mode {
				break 'scope mode.short();
			}
		}

		String::from("???")
	};

	view! { cx,
		<span class="mode_bracket"> "[" </span>
			<span class="mode_name"> {mode_text} </span>
		<span class="mode_bracket"> "]" </span>
	}
}

#[leptos::component]
fn MapName(cx: Scope, info: GetInfo) -> impl IntoView {
	let map_name = move || 'scope: {
		if let Some(info) = info() {
			let map_name = &info.map_name;
			let map_tier = match info.map_tier {
				None => String::from("unknown tier"),
				Some(tier) => format!("T{}", tier as u8),
			};

			break 'scope view! { cx,
				<span class="map_name"> {map_name} </span> " (" {map_tier} ")"
			};
		}

		view! { cx, <><span class="map_name"> "unknown map" </span></> }
	};

	view! { cx,
		{map_name}
	}
}
