mod colors;
mod components;
mod gsi_gui;
pub use gsi_gui::GSIGui;

use crate::config::Config;
use eframe::egui;
use std::process::exit;
use tracing::{error, info, trace};

impl eframe::App for GSIGui {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		components::panel_header(ctx);
		components::panel_center(ctx);
		components::panel_bottom(ctx);
	}

	fn save(&mut self, _storage: &mut dyn eframe::Storage) {
		use std::{fs::File, io::Write};

		let config_path = Config::default_location();
		let config = toml::to_string_pretty(&self.config).expect("Failed to serialize config.");
		let mut config_file = File::create(&config_path).expect("Failed to open config file.");

		if let Err(err) = config_file.write_all(config.as_bytes()) {
			error!(?err, "Failed to save config file.");
			exit(1);
		}

		trace!(path = ?config_path, "Saved config.");
	}

	fn on_exit(&mut self, _glow_ctx: Option<&eframe::glow::Context>) {
		info!("Goodbye.");
	}

	fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
		visuals
			.window_fill()
			.to_normalized_gamma_f32()
	}
}
