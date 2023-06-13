use eframe::egui::{self, TopBottomPanel};

pub fn panel_bottom(ctx: &egui::Context) {
	TopBottomPanel::bottom("panel-bottom").show(ctx, |ui| {
		// TODO
		ui.label("This is the bottom!");
	});
}
