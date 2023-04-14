use {crate::config::Config, eframe::egui::CentralPanel, std::fs::File, tracing::info};

mod client;
pub use client::Client;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
	Main,
	Logs,
}

impl eframe::App for Client {
	fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
		CentralPanel::default().show(ctx, |ui| {
			ui.label("HI");
		});
	}

	fn save(&mut self, _storage: &mut dyn eframe::Storage) {
		use std::io::Write;

		let config_path = Config::find_path().expect("Failed to find config path.");
		let config = tokio::task::block_in_place(|| self.config.blocking_lock().clone());
		let mut config_file = File::create(&config_path).expect("Failed to open config file.");
		let config = toml::to_string_pretty(&config).expect("Failed to serialize config.");

		if let Err(why) = config_file.write_all(config.as_bytes()) {
			panic!("Failed to save config file.\n{why:#?}");
		}

		info!("Successfully wrote config to `{}`.", config_path.display());
	}
}
