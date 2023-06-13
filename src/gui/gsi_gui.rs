use super::{colors, tab::Tab};
use crate::{
	config::Config,
	logger::{LogReceiver, LOG_CHANNEL},
};
use color_eyre::Result;
use eframe::{
	egui::{
		self, Button, CentralPanel, FontData, FontDefinitions, RichText, Style, TextEdit,
		TextStyle, TopBottomPanel, Ui,
	},
	epaint::{FontFamily, FontId},
	HardwareAcceleration, NativeOptions, Theme,
};
// use rfd::FileDialog;
use std::{collections::BTreeMap, path::PathBuf};
use uuid::Uuid;

pub struct GSIGui {
	pub config: Config,
	pub logs: LogReceiver,

	pub current_tab: Tab,
	api_key_buffer: String,
}

impl GSIGui {
	pub const APP_NAME: &str = "Schnose GSI Client";
	pub const DEFAULT_FONT: &str = "Quicksand";
	pub const MONOSPACE_FONT: &str = "Fira Code";
	pub const _DEFAULT_SPACING: f32 = 8.0;

	#[tracing::instrument]
	pub fn init(config_path: Option<PathBuf>) -> Result<()> {
		let config = match config_path {
			None => crate::Config::load(),
			Some(config_path) => crate::Config::load_from_file(config_path),
		}?;

		let api_key_buffer = config
			.schnose_api_key
			.map(|uuid| uuid.to_string())
			.unwrap_or_default();

		let gui = Self {
			config,
			logs: LOG_CHANNEL.subscribe(),
			current_tab: Tab::Main,
			api_key_buffer,
		};

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

	pub fn set_theme(&self, ctx: &egui::Context) {
		catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);
	}
}

// {{{ Center Panel
impl GSIGui {
	pub fn panel_center(&mut self, ctx: &egui::Context) {
		CentralPanel::default().show(ctx, |ui| match self.current_tab {
			Tab::Main => self.render_main(ui),
			Tab::Logs => self.render_logs(ui),
		});
	}
}
// }}}

// {{{ Main Tab
impl GSIGui {
	fn render_main(&mut self, ui: &mut Ui) {
		ui.vertical_centered(|ui| {
			self.render_cfg_prompt(ui);
			self.render_key_prompt(ui);
		});
	}

	fn render_cfg_prompt(&mut self, ui: &mut Ui) {
		let button = Button::new("Select your /csgo/cfg folder").fill(colors::SURFACE2);
		let button = ui.add(button);

		// if button.clicked() {
		// 	self.config.cfg_path = FileDialog::new().pick_folder();
		// }

		let current_folder = self
			.config
			.cfg_path
			.as_ref()
			.map(|path| path.display().to_string())
			.unwrap_or_default();

		button.on_hover_text(format!("Current folder: {current_folder}"));
	}

	fn render_key_prompt(&mut self, ui: &mut Ui) {
		ui.label("Enter your API Key: ");

		TextEdit::singleline(&mut self.api_key_buffer)
			.password(true)
			.show(ui);

		if let Ok(api_key) = Uuid::parse_str(&self.api_key_buffer) {
			match self.config.schnose_api_key.as_mut() {
				None => {
					self.config.schnose_api_key = Some(api_key);
				}

				Some(old_key) => {
					*old_key = api_key;
				}
			};
		}

		if self.api_key_buffer.is_empty() {
			self.config.schnose_api_key = None;
		}
	}
}
// }}}

// {{{ Log Tab
impl GSIGui {
	fn render_logs(&mut self, ui: &mut Ui) {
		ui.label("LOG WINDOW COMING SOONTM");
	}
}
// }}}

// {{{ Header Panel
impl GSIGui {
	pub fn panel_header(&mut self, ctx: &egui::Context) {
		TopBottomPanel::top("panel-header").show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.selectable_value(&mut self.current_tab, Tab::Main, "Main");
				ui.selectable_value(&mut self.current_tab, Tab::Logs, "Logs");
			});

			ui.vertical_centered(|ui| {
				let header_text = RichText::new("Schnose GSI Client")
					.color(colors::POGGERS)
					.heading();

				ui.label(header_text);
			});
		});
	}
}
// }}}

// {{{ Bottom Panel
impl GSIGui {
	pub fn panel_bottom(&mut self, ctx: &egui::Context) {
		TopBottomPanel::bottom("panel-bottom").show(ctx, |ui| {
			// TODO
			ui.label("This is the bottom!");
		});
	}
}
// }}}

// vim: fdm=marker fdl=0
