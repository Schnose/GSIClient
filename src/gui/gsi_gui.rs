use crate::{
	config::Config,
	logger::{LogReceiver, LOG_CHANNEL},
};
use color_eyre::Result;
use eframe::{
	egui::{self, FontData, FontDefinitions, Style, TextStyle},
	epaint::{FontFamily, FontId},
	HardwareAcceleration, NativeOptions, Theme,
};
use std::{collections::BTreeMap, path::PathBuf};

pub struct GSIGui {
	pub config: Config,
	pub logs: LogReceiver,
}

impl GSIGui {
	pub const APP_NAME: &str = "Schnose GSI Client";
	pub const DEFAULT_FONT: &str = "Quicksand";
	pub const MONOSPACE_FONT: &str = "Fira Code";
	pub const DEFAULT_SPACING: f32 = 8.0;

	#[tracing::instrument]
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
				Self::load_fonts(&ctx.egui_ctx);
				Box::new(gui)
			}),
		)
		.expect("Failed to run GUI.");

		Ok(())
	}

	#[tracing::instrument(skip(ctx))]
	fn load_fonts(ctx: &egui::Context) {
		let mut font_definitions = FontDefinitions::default();

		font_definitions.font_data.insert(
			String::from(Self::DEFAULT_FONT),
			FontData::from_static(include_bytes!("../../assets/fonts/quicksand.ttf")),
		);

		font_definitions.font_data.insert(
			String::from(Self::MONOSPACE_FONT),
			FontData::from_static(include_bytes!("../../assets/fonts/firacode.ttf")),
		);

		font_definitions
			.families
			.entry(FontFamily::Proportional)
			.or_default()
			.insert(0, String::from(Self::DEFAULT_FONT));

		font_definitions
			.families
			.entry(FontFamily::Monospace)
			.or_default()
			.insert(0, String::from(Self::MONOSPACE_FONT));

		ctx.set_fonts(font_definitions);
		ctx.set_style(Style {
			text_styles: BTreeMap::from_iter([
				(TextStyle::Heading, FontId::new(36.0, FontFamily::Proportional)),
				(TextStyle::Body, FontId::new(24.0, FontFamily::Proportional)),
				(TextStyle::Button, FontId::new(24.0, FontFamily::Proportional)),
				(TextStyle::Monospace, FontId::new(24.0, FontFamily::Monospace)),
			]),
			..Default::default()
		});
	}
}
