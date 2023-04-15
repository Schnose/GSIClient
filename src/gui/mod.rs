use {
	crate::{colors, config::Config},
	eframe::egui::{CentralPanel, RichText, TopBottomPanel},
	std::fs::File,
	tracing::info,
};

mod client;
pub use client::Client;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
	Main,
	Logs,
}

impl eframe::App for Client {
	fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
		TopBottomPanel::top("header-panel").show(ctx, |ui| {
			ui.add_space(Self::DEFAULT_SPACING);

			ui.horizontal(|ui| {
				ui.selectable_value(&mut self.current_tab, Tab::Main, "Main");
				ui.selectable_value(&mut self.current_tab, Tab::Logs, "Logs");
			});

			ui.add_space(Self::DEFAULT_SPACING);
			ui.separator();
			ui.add_space(Self::DEFAULT_SPACING);

			let header = RichText::new("Schnose GSI Client")
				.color(colors::POGGERS)
				.heading();

			ui.vertical_centered(|ui| ui.label(header));

			ui.add_space(Self::DEFAULT_SPACING);
		});

		CentralPanel::default().show(ctx, |ui| {
			match self.current_tab {
				Tab::Main => self.render_main(ui),
				Tab::Logs => self.render_logs(ui),
			};
		});

		self.notifications.show(ctx);

		TopBottomPanel::bottom("footer-panel").show(ctx, |ui| {
			ui.add_space(Self::DEFAULT_SPACING);

			ui.vertical_centered(|ui| self.render_status(ui));

			ui.add_space(Self::DEFAULT_SPACING);
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
