mod colors;
mod tab;

mod gsi_gui;
pub use gsi_gui::GSIGui;

use eframe::egui;
use schnose_gsi_client::Config;
use tracing::{error, info, trace};

impl eframe::App for GSIGui {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		self.set_theme(ctx);
		self.panel_header(ctx);
		self.panel_center(ctx);
		self.panel_bottom(ctx);
	}

	fn save(&mut self, _storage: &mut dyn eframe::Storage) {
		use std::{fs::File, io::Write};

		let config_path = Config::default_location();
		let config = toml::to_string_pretty(&self.config).expect("Failed to serialize config.");
		let mut config_file = match File::create(&config_path) {
			Ok(config_file) => config_file,
			Err(error) => {
				error!(?error, "Failed to open config file.");
				return;
			}
		};

		if let Err(err) = config_file.write_all(config.as_bytes()) {
			error!(?err, "Failed to save config file.");
			return;
		}

		trace!(path = ?config_path, "Saved config.");
	}

	fn on_exit(&mut self, _glow_ctx: Option<&eframe::glow::Context>) {
		self.stop_server()
			.expect("Failed to shutdown GSI server");

		info!("Goodbye.");
	}

	fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
		visuals
			.window_fill()
			.to_normalized_gamma_f32()
	}
}
