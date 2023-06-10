use crate::{
	config::Config,
	logger::{LogReceiver, LOG_CHANNEL},
};
use color_eyre::Result;
use eframe::{HardwareAcceleration, NativeOptions, Theme};
use std::path::PathBuf;

pub struct GSIGui {
	pub config: Config,
	pub logs: LogReceiver,
}

impl GSIGui {
	const APP_NAME: &str = "schnose-gsi-client";

	pub fn init(config_path: Option<PathBuf>) -> Result<()> {
		let config = match config_path {
			None => crate::Config::load(),
			Some(config_path) => crate::Config::load_from_file(config_path),
		}?;

		let gui = Self { config, logs: LOG_CHANNEL.subscribe() };

		let native_options = NativeOptions {
			always_on_top: false,
			decorated: true,
			fullscreen: false,
			drag_and_drop_support: true,
			resizable: true,
			transparent: true,
			mouse_passthrough: false,
			vsync: true,
			hardware_acceleration: HardwareAcceleration::Preferred,
			follow_system_theme: false,
			default_theme: Theme::Dark,
			centered: true,
			..Default::default()
		};

		eframe::run_native(
			Self::APP_NAME,
			native_options,
			Box::new(|ctx| {
				catppuccin_egui::set_theme(&ctx.egui_ctx, catppuccin_egui::MOCHA);
				Box::new(gui)
			}),
		)
		.expect("Failed to run GUI.");

		Ok(())
	}
}
