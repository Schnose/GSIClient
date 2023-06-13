use eframe::egui::{self, CentralPanel};

pub fn panel_center(ctx: &egui::Context) {
	CentralPanel::default().show(ctx, |ui| {
		// TODO
		ui.label("This is the center panel!");
	});
}
