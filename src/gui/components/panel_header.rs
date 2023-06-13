use crate::gui::colors;
use eframe::egui::{self, RichText, TopBottomPanel};

pub fn panel_header(ctx: &egui::Context) {
	TopBottomPanel::top("panel-header").show(ctx, |ui| {
		ui.horizontal(|ui| {
			// TODO
			ui.label("This is a tab bar.");
		});

		ui.vertical_centered(|ui| {
			let header_text = RichText::new("Schnose GSI Client")
				.color(colors::POGGERS)
				.heading();

			ui.label(header_text);
		});
	});
}
