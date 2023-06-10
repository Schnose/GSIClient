mod args;
mod config;
mod gui;
mod logger;

use args::Args;
use gui::GSIGui;
use std::process::exit;
use color_eyre::Result;
use tracing::error;

pub(crate) use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;

	let args = args::get();

	// Initialize logging
	setup_tracing(&args);

	if let Err(err) = GSIGui::init(args.config_path) {
		error!("Failed to run GUI.");
		error!("{err:#?}");
		exit(1);
	}

	Ok(())
}

fn setup_tracing(Args { log_level, .. }: &Args) {
	use logger::Logger;
	use time::macros::format_description;
	use tracing_subscriber::fmt::time::UtcTime;

	let log_level = std::env::var("RUST_LOG")
		.unwrap_or_else(|_| format!("ERROR,schnose_gsi_client={log_level}"));

	let timer_format = format_description!("[[[year]-[month]-[day] | [hour]:[minute]:[second]]");
	let timer = UtcTime::new(timer_format);

	let writer = Logger::new();

	tracing_subscriber::fmt()
		.pretty()
		.with_env_filter(log_level)
		.with_file(true)
		.with_line_number(true)
		.with_timer(timer)
		.with_writer(writer)
		.init();
}
