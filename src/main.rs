mod args;
mod gui;
mod logger;

use args::Args;
use color_eyre::Result;
use gui::GSIGui;
use std::process::exit;
use tracing::error;

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;

	let args = args::get();

	// Initialize logging
	setup_tracing(&args);

	if let Err(err) = GSIGui::init(args.config_path) {
		error!(?err, "failed to run GUI");
		exit(1);
	}

	Ok(())
}

fn setup_tracing(Args { log_level, .. }: &Args) {
	use logger::Logger;
	use time::macros::format_description;
	use tracing_subscriber::{
		fmt::{format::FmtSpan, time::UtcTime},
		layer::SubscriberExt,
		util::SubscriberInitExt,
		EnvFilter, Layer,
	};

	let log_level = std::env::var("RUST_LOG")
		.unwrap_or_else(|_| format!("ERROR,schnose_gsi_client={log_level}"));

	let timer_format = format_description!("[[[year]-[month]-[day] | [hour]:[minute]:[second]]");
	let writer = Logger::new();

	macro_rules! subscriber {
		() => {
			tracing_subscriber::fmt::layer()
				.with_file(true)
				.with_line_number(true)
				.with_timer(UtcTime::new(timer_format))
				.with_span_events(FmtSpan::FULL)
				.pretty()
		};
	}

	tracing_subscriber::registry()
		.with(subscriber!().with_filter(EnvFilter::from(log_level)))
		.with(
			subscriber!()
				.json()
				.with_writer(writer)
				.with_filter(EnvFilter::from(
					"schnose_gsi:INFO,gokz_rs=INFO,schnose_gsi_client=TRACE",
				)),
		)
		.init();
}
