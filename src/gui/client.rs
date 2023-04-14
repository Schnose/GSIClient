use {
	super::Tab,
	crate::{colors, config::Config, logger::LogReceiver},
	eframe::{
		egui::{style::Selection, FontData, FontDefinitions, Style, TextStyle, Visuals},
		epaint::{FontFamily, FontId},
		CreationContext,
	},
	eframe::{HardwareAcceleration, NativeOptions, Theme},
	std::{collections::BTreeMap, sync::Arc},
	tokio::{sync::Mutex, task::JoinHandle},
};

pub struct Client {
	pub config: Arc<Mutex<Config>>,
	pub logger: Option<LogReceiver>,
	pub current_tab: Tab,
	pub state: Arc<Mutex<Option<crate::gsi::State>>>,
	pub gsi_handle: Option<schnose_gsi::ServerHandle>,
	pub axum_handle: Option<JoinHandle<()>>,
}

impl Client {
	pub const APP_NAME: &str = "schnose-gsi-client";
	pub const DEFAULT_FONT: &str = "Quicksand";
	pub const MONOSPACE_FONT: &str = "Fira Code";

	#[tracing::instrument]
	pub async fn init(config: Config, logger: Option<LogReceiver>) {
		let client = Self {
			config: Arc::new(Mutex::new(config)),
			logger,
			current_tab: Tab::Main,
			state: Arc::new(Mutex::new(None)),
			gsi_handle: None,
			axum_handle: None,
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
				Self::load_fonts(ctx);
				Self::load_visuals(ctx);
				Box::new(client)
			}),
		)
		.expect("Failed to run GUI.")
	}

	#[tracing::instrument(skip(ctx))]
	fn load_fonts(ctx: &CreationContext) {
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

		ctx.egui_ctx.set_fonts(font_definitions);
		ctx.egui_ctx.set_style(Style {
			text_styles: BTreeMap::from_iter([
				(TextStyle::Heading, FontId::new(36.0, FontFamily::Proportional)),
				(TextStyle::Body, FontId::new(24.0, FontFamily::Proportional)),
				(TextStyle::Button, FontId::new(24.0, FontFamily::Proportional)),
				(TextStyle::Monospace, FontId::new(24.0, FontFamily::Monospace)),
			]),
			..Default::default()
		});
	}

	#[tracing::instrument(skip(ctx))]
	fn load_visuals(ctx: &CreationContext) {
		ctx.egui_ctx.set_visuals(Visuals {
			dark_mode: true,
			override_text_color: Some(colors::TEXT),
			selection: Selection {
				bg_fill: colors::SURFACE2,
				..Default::default()
			},
			hyperlink_color: colors::MAUVE,
			faint_bg_color: colors::MANTLE,
			extreme_bg_color: colors::CRUST,
			code_bg_color: colors::MANTLE,
			warn_fg_color: colors::PEACH,
			error_fg_color: colors::RED,
			window_fill: colors::BASE,
			panel_fill: colors::MANTLE,
			button_frame: true,
			slider_trailing_fill: true,
			..Default::default()
		});
	}
}
