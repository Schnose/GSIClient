mod gsi_gui;
use eframe::egui::CentralPanel;
pub use gsi_gui::GSIGui;

use crate::config::Config;
use std::process::exit;
use tracing::{error, info};

impl eframe::App for GSIGui {
	fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
		CentralPanel::default().show(ctx, |ui| {
			ui.label("Hello, world!");
		});
	}

	fn save(&mut self, _storage: &mut dyn eframe::Storage) {
		use std::{fs::File, io::Write};

		let config_path = Config::default_location();
		let config = toml::to_string_pretty(&self.config).expect("Failed to serialize config.");
		let mut config_file = File::create(&config_path).expect("Failed to open config file.");

		if let Err(err) = config_file.write_all(config.as_bytes()) {
			error!("Failed to save config file.");
			error!("{err:#?}");
			exit(1);
		}

		info!("Wrote config to `{}`.", config_path.display());
	}

	fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
		info!("Goodbye.");
	}

	fn clear_color(&self, visuals: &eframe::egui::Visuals) -> [f32; 4] {
		visuals
			.window_fill()
			.to_normalized_gamma_f32()
	}
}
