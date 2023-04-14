use {
	super::Tab,
	crate::{
		colors,
		config::Config,
		logger::{Log, LogReceiver},
	},
	chrono::Utc,
	eframe::{
		egui::{
			style::Selection, FontData, FontDefinitions, RichText, ScrollArea, Style, TextStyle,
			Ui, Visuals,
		},
		epaint::{FontFamily, FontId},
		CreationContext,
	},
	eframe::{HardwareAcceleration, NativeOptions, Theme},
	rfd::FileDialog,
	std::{collections::BTreeMap, fs::File, sync::Arc},
	tokio::{sync::Mutex, task::JoinHandle},
	tracing::{error, info, warn},
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

	pub fn render_main(&self, ui: &mut Ui) {
		ui.label("THIS IS MAIN");
	}

	pub fn render_logs(&mut self, ui: &mut Ui) {
		if self.logger.is_some() {
			// TODO: put this below the scroll area
			self.save_logs(ui);

			ui.add_space(8.0);
			ui.separator();
			ui.add_space(8.0);
		}

		let Some(logger) = &mut self.logger else {
			ui.vertical_centered(|ui| ui.colored_label(colors::RED, "Logs are displayed on STDOUT."));
			return;
		};

		let Some(logs) = logger.current() else {
			return;
		};

		let logs = Log::from_slice(&logs);

		ScrollArea::new([true; 2])
			.auto_shrink([false; 2])
			.stick_to_bottom(true)
			.show_rows(ui, 8.0, logs.len(), |ui, range| {
				logs.into_iter()
					.skip(range.start)
					.take(range.len())
					.for_each(|log| {
						ui.horizontal(|ui| {
							ui.add_space(4.0);

							ui.label(log.timestamp);

							ui.add_space(4.0);
							ui.separator();
							ui.add_space(4.0);

							ui.label(log.level);

							ui.add_space(4.0);
							ui.separator();
							ui.add_space(4.0);

							ui.label(log.message);
							ui.add_space(4.0);
						});
					});
			});

		ui.add_space(8.0);

		// FIXME: this is not being displayed for some reason
		ui.label("HI");
	}

	pub fn render_status(&self, ui: &mut Ui) {
		let status = if self.gsi_handle.is_some() && self.axum_handle.is_some() {
			RichText::new("Running").color(colors::GREEN)
		} else {
			RichText::new("Stopped").color(colors::RED)
		}
		.heading();

		ui.label(status);
	}

	pub fn save_logs(&mut self, ui: &mut Ui) {
		use std::io::Write;

		if !ui
			.button(RichText::new("Save logs").color(colors::PEACH))
			.clicked()
		{
			return;
		}

		let timestamp = Utc::now().format("%Y%m%d%H%M%S");
		let file_name = format!("{timestamp}-schnose-gsi-client.log");

		let Some(log_path) = FileDialog::new().set_file_name(&file_name).save_file() else {
			return;
		};

		let mut file = match File::create(&log_path) {
			Ok(file) => file,
			Err(why) => return error!("Failed to create log file: {why:#?}"),
		};

		let Some(logger) = &mut self.logger else {
			return error!("This UI should only be rendered if a logger is present.");
		};

		let Some(logs) = logger.current() else {
			return warn!("Cannot save empty logs.");
		};

		let log_path = log_path.display();

		match file.write_all(&logs) {
			Ok(()) => info!("Wrote logs to `{log_path}`."),
			Err(why) => error!("Failed to write logs to `{log_path}`: {why:#?}"),
		}
	}
}
