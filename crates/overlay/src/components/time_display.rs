#![allow(unused_braces)]

use crate::{util::format_time, GetInfo};
use leptos::{view, IntoView, Scope};

#[leptos::component]
pub fn TimeDisplay(cx: Scope, info: GetInfo) -> impl IntoView {
	let title = move |tp: bool| match tp {
		true => view! { cx, <span class="tp"> "TP" </span> },
		false => view! { cx, <span class="pro"> "PRO" </span> },
	};

	let colon = move || view! { cx, <span class="mode_bracket"> ": " </span> };

	view! { cx,
		<div>
			{title(true)} {colon} <WR info tp=true /> <PB info tp=true />
		</div>

		<div>
			{title(false)} {colon} <WR info tp=false /> <PB info tp=false />
		</div>
	}
}

#[leptos::component]
fn WR(cx: Scope, info: GetInfo, tp: bool) -> impl IntoView {
	let wr_time = move || 'scope: {
		if let Some(ref info) = info() {
			let record = match tp {
				true => info.tp_wr.as_ref(),
				false => info.pro_wr.as_ref(),
			};

			if let Some(record) = record {
				let time = format_time(record.time);
				let player_name = &record.player_name;

				break 'scope format!("{time} by {player_name}");
			}
		}

		String::from("no WR")
	};

	view! { cx, <span> {wr_time} </span> }
}

#[leptos::component]
fn PB(cx: Scope, info: GetInfo, tp: bool) -> impl IntoView {
	'scope: {
		if let Some(ref info) = info() {
			let wr_seconds = match tp {
				true => info.tp_wr.as_ref(),
				false => info.pro_wr.as_ref(),
			}
			.expect("If the player has a PB, there must be a WR.")
			.time;

			let record = match tp {
				true => info.tp_wr.as_ref(),
				false => info.pro_wr.as_ref(),
			};

			if let Some(record) = record {
				break 'scope if wr_seconds < record.time {
					let diff = wr_seconds - record.time;
					let diff = format_time(diff);
					let text = format!(" (+{diff})");

					view! { cx,
						<span class="time_diff"> {text} </span>
					}
				} else {
					view! { cx,
						<span class="wr_by_me"> " (WR by me)" </span>
					}
				};
			}
		}

		view! { cx, <span></span> }
	}
}
