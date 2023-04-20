#![windows_subsystem = "windows"]

use {
	crate::{config::Config, gui::Client},
	clap::Parser,
	color_eyre::{eyre::Context, Result},
	std::{path::PathBuf, sync::Arc},
	tracing::Level,
	tracing_subscriber::fmt::format::FmtSpan,
};

mod colors;
mod config;
mod gsi;
mod gui;
mod logger;
mod server;

#[derive(Debug, Parser)]
struct Args {
	/// Send logs to STDOUT instead of a tab in the GUI.
	#[arg(long = "logs")]
	#[clap(default_value = "false")]
	log_to_stdout: bool,

	/// RUST_LOG=DEBUG
	#[arg(long)]
	#[clap(default_value = "false")]
	debug: bool,

	/// Use a custom config file.
	#[arg(short, long = "config")]
	config_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;
	let args = Args::parse();

	let subscriber = tracing_subscriber::fmt()
		.compact()
		.with_file(true)
		.with_line_number(true)
		.with_span_events(FmtSpan::NEW)
		.with_max_level(match args.debug {
			true => Level::DEBUG,
			false => Level::INFO,
		});

	let logger = if args.log_to_stdout {
		subscriber.init();
		None
	} else {
		let (log_sender, log_receiver) = logger::new();

		subscriber
			.json()
			.with_max_level(Level::DEBUG)
			.with_writer(Arc::new(log_sender))
			.init();

		Some(log_receiver)
	};

	let config = match args.config_path {
		None => Config::load().context("Failed to load config file.")?,
		Some(config_path) => {
			let config_file =
				std::fs::read_to_string(config_path).context("Failed to read config file.")?;
			toml::from_str(&config_file).context("Failed to deserialize config file.")?
		}
	};

	Client::init(config, logger).await;

	Ok(())
}
