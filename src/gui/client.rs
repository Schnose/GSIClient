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
			style::Selection, Align, Button, FontData, FontDefinitions, Layout, RichText, Style,
			TextEdit, TextStyle, Ui, Visuals,
		},
		epaint::{FontFamily, FontId},
		CreationContext,
	},
	eframe::{HardwareAcceleration, NativeOptions, Theme},
	egui_extras::{Column, TableBuilder},
	egui_notify::Toasts,
	rfd::FileDialog,
	std::{collections::BTreeMap, fs::File, sync::Arc, time::Duration},
	tokio::{
		sync::{broadcast, Mutex},
		task::JoinHandle,
	},
	tracing::{error, info, warn},
	uuid::Uuid,
};

pub struct Client {
	pub config: Arc<Mutex<Config>>,
	pub logger: Option<LogReceiver>,
	pub current_tab: Tab,
	pub notifications: Toasts,
	pub api_key_prompt: String,
	pub gsi_handle: Option<schnose_gsi::ServerHandle>,
	pub axum_handle: Option<JoinHandle<()>>,
}

impl Client {
	pub const APP_NAME: &str = "schnose-gsi-client";
	pub const DEFAULT_FONT: &str = "Quicksand";
	pub const MONOSPACE_FONT: &str = "Fira Code";
	pub const DEFAULT_SPACING: f32 = 8.0;
	pub const NOTIFICATION_DURATION: Option<Duration> = Some(Duration::from_secs(3));

	#[tracing::instrument]
	pub async fn init(config: Config, logger: Option<LogReceiver>) {
		let api_key_prompt = config
			.schnose_api_key
			.map(|uuid| uuid.to_string())
			.unwrap_or_default();

		let client = Self {
			config: Arc::new(Mutex::new(config)),
			logger,
			current_tab: Tab::Main,
			notifications: Toasts::default(),
			api_key_prompt,
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

	const fn server_running(&self) -> bool {
		self.gsi_handle.is_some() && self.axum_handle.is_some()
	}

	fn spacing(ui: &mut Ui) {
		ui.add_space(Self::DEFAULT_SPACING);
	}

	pub fn render_main(&mut self, ui: &mut Ui) {
		ui.vertical_centered(|ui| {
			self.render_cfg_prompt(ui);
			self.render_key_prompt(ui);
		});

		Self::spacing(ui);
		ui.separator();
		Self::spacing(ui);

		ui.vertical_centered(|ui| self.render_run_button(ui));
		Self::spacing(ui);
	}

	fn render_cfg_prompt(&mut self, ui: &mut Ui) {
		let button = ui.add(Button::new("Select your /csgo/cfg folder").fill(colors::SURFACE2));

		let config = &mut *tokio::task::block_in_place(|| self.config.blocking_lock());

		if button.clicked() {
			if let Some(new_cfg_path) = FileDialog::new().pick_folder() {
				config.csgo_cfg_path = Some(new_cfg_path);
			}
		}

		button.on_hover_text(format!(
			"Current folder: {}",
			config
				.csgo_cfg_path
				.as_ref()
				.map(|path| path.display().to_string())
				.unwrap_or_default()
		));
	}

	fn render_key_prompt(&mut self, ui: &mut Ui) {
		ui.label("Enter your API Key: ");

		let config = &mut *tokio::task::block_in_place(|| self.config.blocking_lock());

		TextEdit::singleline(&mut self.api_key_prompt)
			.password(true)
			.show(ui);

		if let Ok(new_key) = Uuid::parse_str(&self.api_key_prompt) {
			match config.schnose_api_key.as_mut() {
				None => config.schnose_api_key = Some(new_key),
				Some(old_key) => *old_key = new_key,
			};
		} else if self.api_key_prompt.is_empty() {
			config.schnose_api_key = None;
		}
	}

	fn render_run_button(&mut self, ui: &mut Ui) {
		if self.server_running() {
			let stop_text = RichText::new("Stop GSI Server").color(colors::RED);
			let stop_button = ui.add(Button::new(stop_text).fill(colors::SURFACE2));
			if stop_button.clicked() {
				self.stop_server()
			}
		} else {
			let start_text = RichText::new("Start GSI Server").color(colors::GREEN);
			let start_button = ui.add(Button::new(start_text).fill(colors::SURFACE2));

			if start_button.clicked() {
				{
					let config = tokio::task::block_in_place(|| self.config.blocking_lock());

					let has_path = match &config.csgo_cfg_path {
						None => false,
						Some(path) if path.as_os_str().is_empty() => false,
						_ => true,
					};

					if !has_path {
						self.notifications
							.error("You need to enter a cfg path before you can start the server.")
							.set_duration(Self::NOTIFICATION_DURATION)
							.set_closable(true);
						return;
					}
				}

				self.run_server();
			}
		}
	}

	fn run_server(&mut self) {
		let (state_sender, state_receiver) = broadcast::channel(64);

		self.gsi_handle = match crate::gsi::run(state_sender, Arc::clone(&self.config)) {
			Ok(handle) => {
				self.notifications
					.info("Starting GSI Server...")
					.set_duration(Self::NOTIFICATION_DURATION);
				Some(handle)
			}
			Err(why) => {
				self.notifications
					.error(format!("{why}"))
					.set_duration(Self::NOTIFICATION_DURATION);
				return;
			}
		};

		self.axum_handle = Some(tokio::spawn(crate::server::run(state_receiver)));
		self.notifications
			.info("Starting HTTP Server...")
			.set_duration(Self::NOTIFICATION_DURATION);
	}

	fn stop_server(&mut self) {
		if let Some(handle) = self.axum_handle.take() {
			handle.abort();
			self.notifications
				.info("Stopping HTTP Server...")
				.set_duration(Self::NOTIFICATION_DURATION);
		}

		if let Some(handle) = self.gsi_handle.take() {
			handle.abort();
			self.notifications
				.info("Stopping GSI Server...")
				.set_duration(Self::NOTIFICATION_DURATION);
		}
	}

	pub fn render_logs(&mut self, ui: &mut Ui) {
		let Some(logger) = &mut self.logger else {
			ui.vertical_centered(|ui| ui.colored_label(colors::RED, "Logs are displayed on STDOUT."));
			return;
		};

		let logs = logger.current();
		let logs = Log::from_slice(&logs);

		let button = {
			let mut button = None;

			ui.horizontal(|ui| {
				self.save_logs(ui);
				let jump_button = Button::new("Go to bottom").fill(colors::SURFACE0);
				button = Some(ui.add(jump_button));
			});

			ui.add_space(Self::DEFAULT_SPACING);
			ui.separator();
			ui.add_space(Self::DEFAULT_SPACING);

			button.unwrap()
		};

		let mut table = TableBuilder::new(ui)
			.striped(true)
			.resizable(false)
			.stick_to_bottom(true)
			.cell_layout(Layout::left_to_right(Align::TOP))
			.column(Column::auto())
			.column(Column::auto())
			.column(Column::remainder())
			.vscroll(true)
			.min_scrolled_height(0.0);

		if button.clicked() {
			table = table.scroll_to_row(logs.len(), None);
		}

		table.body(|body| {
			let heights = logs
				.iter()
				.map(|log| {
					let lines = log.message.text().lines().count();
					lines as f32 * Self::DEFAULT_SPACING * 4.0
				})
				.collect::<Vec<_>>();

			body.heterogeneous_rows(heights.into_iter(), |idx, mut row| {
				if let Some(log) = logs.get(idx).cloned() {
					row.col(|ui| {
						ui.label(log.timestamp);
					});

					row.col(|ui| {
						ui.label(log.level);
					});

					row.col(|ui| {
						ui.label(log.message);
					});
				}
			});
		});
	}

	pub fn render_status(&self, ui: &mut Ui) {
		if self.server_running() {
			ui.scope(|ui| {
				ui.style_mut().wrap = Some(true);
				ui.label(RichText::new("Running").color(colors::GREEN));
				ui.hyperlink_to(
					"Open Overlay",
					format!("http://localhost:{}", crate::server::PORT),
				);
			});
		} else {
			ui.label(RichText::new("Stopped").color(colors::RED));
		}
	}

	fn save_logs(&mut self, ui: &mut Ui) {
		use std::io::Write;

		let text = RichText::new("Save logs").color(colors::TEXT);
		let button = Button::new(text).fill(colors::SURFACE0);

		if !ui.add(button).clicked() {
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

		let logs = logger.current();
		let log_path = log_path.display();

		match file.write_all(&logs) {
			Ok(()) => info!("Wrote logs to `{log_path}`."),
			Err(why) => error!("Failed to write logs to `{log_path}`: {why:#?}"),
		}
	}
}
