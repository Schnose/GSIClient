use super::{colors, tab::Tab};
use crate::logger::{
	logs::{self, Log},
	LogReceiver, LOG_CHANNEL,
};
use color_eyre::{eyre::eyre, Result};
use eframe::{
	egui::{
		self, Button, CentralPanel, FontData, FontDefinitions, Layout, RichText, Style, TextEdit,
		TextStyle, TopBottomPanel, Ui,
	},
	emath::Align,
	epaint::{FontFamily, FontId},
	HardwareAcceleration, NativeOptions, Theme,
};
use egui_extras::{Column, TableBuilder};
use rfd::FileDialog;
use schnose_gsi_client::Config;
use schnose_gsi_client_common::GameInfo;
use std::{
	collections::BTreeMap,
	path::PathBuf,
	sync::{Arc, Mutex, MutexGuard},
};
use tokio::{sync::broadcast, task::JoinHandle};
use tracing::{error, trace};
use uuid::Uuid;

pub struct GSIGui {
	pub config: Arc<Mutex<Config>>,
	pub logs: LogReceiver,
	pub log_buf: Vec<Log>,

	pub current_tab: Tab,
	api_key_buffer: String,
	pub running: bool,

	pub gsi_receiver: Option<broadcast::Receiver<GameInfo>>,
	current_info: Option<GameInfo>,
	gsi_handle: Option<schnose_gsi::ServerHandle>,

	axum_handle: Option<JoinHandle<()>>,
}

impl GSIGui {
	pub const APP_NAME: &str = "Schnose GSI Client";
	pub const DEFAULT_FONT: &str = "Quicksand";
	pub const MONOSPACE_FONT: &str = "Fira Code";
	pub const DEFAULT_SPACING: f32 = 8.0;
	pub const MAX_LOGS: usize = 512;

	#[tracing::instrument(name = "Initializing GUI")]
	pub fn init(config_path: Option<PathBuf>) -> Result<()> {
		let config = match config_path {
			None => Config::load(),
			Some(config_path) => Config::load_from_file(config_path),
		}?;

		let api_key_buffer = config
			.schnose_api_key
			.map(|uuid| uuid.to_string())
			.unwrap_or_default();

		let gui = Self {
			config: Arc::new(Mutex::new(config)),
			logs: LOG_CHANNEL.subscribe(),
			log_buf: Vec::with_capacity(Self::MAX_LOGS * 2),
			current_tab: Tab::Main,
			api_key_buffer,
			running: false,
			gsi_receiver: None,
			current_info: None,
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
				Self::load_fonts(&ctx.egui_ctx);
				Box::new(gui)
			}),
		)
		.expect("Failed to run GUI.");

		Ok(())
	}

	#[tracing::instrument(skip(ctx), name = "Loading fonts")]
	fn load_fonts(ctx: &egui::Context) {
		let mut font_definitions = FontDefinitions::default();

		font_definitions.font_data.insert(
			String::from(Self::DEFAULT_FONT),
			FontData::from_static(include_bytes!("../../../assets/fonts/quicksand.ttf")),
		);

		font_definitions.font_data.insert(
			String::from(Self::MONOSPACE_FONT),
			FontData::from_static(include_bytes!("../../../assets/fonts/firacode.ttf")),
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

	pub fn config(&self) -> Result<MutexGuard<Config>> {
		self.config
			.lock()
			.map_err(|err| eyre!("Failed to acquire mutex guard: {err:?}"))
	}

	fn start_server(&mut self) -> Result<()> {
		let (sender, receiver) = broadcast::channel(1);
		let receiver2 = sender.subscribe();

		self.gsi_receiver = Some(receiver);
		self.gsi_handle = Some(schnose_gsi_client::make_server(sender, Arc::clone(&self.config))?);
		self.axum_handle = Some(tokio::spawn(crate::server::make_server(receiver2)));

		Ok(())
	}

	pub fn stop_server(&mut self) -> Result<()> {
		if let Some(handle) = self.gsi_handle.take() {
			handle.abort();
		}

		if let Some(handle) = self.axum_handle.take() {
			handle.abort();
		}

		Ok(())
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
			if let Err(error) = self.render_cfg_prompt(ui) {
				error!(?error, "Failed to render cfg prompt");
			}

			ui.add_space(Self::DEFAULT_SPACING);

			if let Err(error) = self.render_key_prompt(ui) {
				error!(?error, "Failed to render API key prompt");
			}

			ui.add_space(Self::DEFAULT_SPACING);

			if let Err(error) = self.render_server_button(ui) {
				error!(?error, "Failed to render Start/Stop button");
			}

			ui.add_space(Self::DEFAULT_SPACING);

			if let Err(error) = self.render_game_info(ui) {
				error!(?error, "Failed to render cfg prompt");
			}
		});
	}

	fn render_cfg_prompt(&mut self, ui: &mut Ui) -> Result<()> {
		let button = Button::new("Select your /csgo/cfg folder").fill(colors::SURFACE2);
		let button = ui.add(button);

		if button.clicked() {
			let mut dialog = FileDialog::new();

			if let Some(ref path) = self.config()?.cfg_path {
				dialog = dialog.set_directory(path);
			}

			// NOTE: Don't just assign `self.config.cfg_path` directly! That would override an
			// existing path if the user cancels the dialog.
			if let Some(path) = dialog.pick_folder() {
				self.config()?.cfg_path = Some(path);
			}
		}

		let current_folder = self
			.config()?
			.cfg_path
			.as_ref()
			.map(|path| path.display().to_string())
			.unwrap_or_default();

		button.on_hover_text(format!("Current folder: {current_folder}"));

		Ok(())
	}

	fn render_key_prompt(&mut self, ui: &mut Ui) -> Result<()> {
		ui.label("Enter your API Key: ");

		TextEdit::singleline(&mut self.api_key_buffer)
			.password(true)
			.show(ui);

		if let Ok(api_key) = Uuid::parse_str(&self.api_key_buffer) {
			match self.config()?.schnose_api_key.as_mut() {
				None => {
					self.config()?.schnose_api_key = Some(api_key);
				}

				Some(old_key) => {
					*old_key = api_key;
				}
			};
		}

		if self.api_key_buffer.is_empty() {
			self.config()?.schnose_api_key = None;
		}

		Ok(())
	}

	fn render_server_button(&mut self, ui: &mut Ui) -> Result<()> {
		if self.running {
			let text = RichText::new("Stop GSI server").color(colors::RED);
			let button = Button::new(text).fill(colors::SURFACE2);

			if ui.add(button).clicked() {
				self.stop_server()?;
				self.running = false;
			}
		} else {
			let text = RichText::new("Start GSI server").color(colors::GREEN);
			let button = Button::new(text).fill(colors::SURFACE2);

			if ui.add(button).clicked() {
				{
					let config = self.config()?;

					let valid_path =
						matches!(config.cfg_path, Some(ref path) if !path.as_os_str().is_empty());

					if !valid_path {
						// TODO: send notification
					}
				}

				self.start_server()?;
				self.running = true;
			}
		}

		Ok(())
	}

	fn render_game_info(&mut self, ui: &mut Ui) -> Result<()> {
		if let Some(Ok(info)) = self
			.gsi_receiver
			.as_mut()
			.map(|recv| recv.try_recv())
		{
			trace!(?info, "new game info");
			self.current_info = Some(info);
		}

		let Some(ref info) = self.current_info else {
			return Ok(());
		};

		ui.vertical_centered_justified(|ui| {
			ui.colored_label(
				colors::SAPPHIRE,
				format!(
					"Player: {}{}",
					info.player_name,
					info.steam_id
						.map(|steam_id| format!(" ({steam_id})"))
						.unwrap_or_default()
				),
			);

			ui.colored_label(
				colors::SAPPHIRE,
				format!(
					"Mode: {}",
					info.mode
						.map(|mode| mode.to_string())
						.unwrap_or_else(|| String::from("None")),
				),
			);

			ui.colored_label(
				colors::SAPPHIRE,
				format!(
					"Map: {}{}",
					info.map_name,
					info.map_tier
						.map(|tier| format!(" (T{})", tier as u8))
						.unwrap_or_default()
				),
			);
		});

		Ok(())
	}
}
// }}}

// {{{ Log Tab
impl GSIGui {
	fn render_logs(&mut self, ui: &mut Ui) {
		while let Ok(log) = self.logs.try_recv() {
			self.log_buf.push(log);
		}

		// Truncate old logs
		if let Some(overflow) = self
			.log_buf
			.len()
			.checked_sub(Self::MAX_LOGS)
		{
			self.log_buf.drain(..overflow);
		}

		let mut button = None;

		ui.horizontal(|ui| {
			// TODO: button to save logs to a file
			button = Some(ui.add(Button::new("Jump to bottom").fill(colors::SURFACE0)));
		});

		// TODO: do this without unwrapping?
		let button = button.unwrap();

		let mut table = TableBuilder::new(ui)
			.resizable(false)
			.stick_to_bottom(true)
			.cell_layout(Layout::left_to_right(Align::TOP))
			.column(Column::auto())
			.column(Column::auto())
			.column(Column::auto())
			.column(Column::auto())
			.column(Column::remainder())
			.vscroll(true)
			.min_scrolled_height(0.0);

		if button.clicked() {
			table = table.scroll_to_row(usize::MAX, None);
		}

		table.body(|body| {
			let heights = self.log_buf.iter().map(|log| {
				let lines = log
					.fields
					.get("message")
					.map(|value| value.to_string())
					.unwrap_or_default();

				lines.lines().count() as f32 * 32.0
			});

			body.heterogeneous_rows(heights, |idx, mut row| {
				let Some(log) = self.log_buf.get(idx) else {
					return;
				};

				row.col(|ui| {
					let text = RichText::new(format!("{:25}", log.timestamp)).color(colors::MAUVE);
					ui.label(text);
				});

				row.col(|ui| {
					let text = RichText::new(format!("{:5?}", log.level)).color(match log.level {
						logs::Level::Trace => colors::TEAL,
						logs::Level::Debug => colors::BLUE,
						logs::Level::Info => colors::GREEN,
						logs::Level::Warn => colors::YELLOW,
						logs::Level::Error => colors::RED,
					});

					ui.separator();
					ui.label(text);
				});

				row.col(|ui| {
					let text = log
						.fields
						.get("message")
						.map(|value| value.to_string());

					if let Some(text) = text {
						ui.separator();
						ui.label(text);
					}
				});

				row.col(|ui| {
					let text = log
						.rest
						.get("target")
						.map(|value| value.to_string())
						.map(|text| RichText::new(text).color(colors::MAROON));

					if let Some(text) = text {
						ui.separator();
						ui.label(text);
					}
				});

				row.col(|ui| {
					if log.fields.len() <= 1 {
						return;
					}

					ui.label(
						RichText::new("{")
							.color(colors::SURFACE1)
							.italics(),
					);

					for (field, value) in log
						.fields
						.iter()
						.filter(|(k, _)| *k != "message")
					{
						let value = value
							.as_str()
							.map_or_else(|| value.to_string(), |value| value.to_owned());

						let text = RichText::new(format!("{field} = {value}"))
							.color(colors::SURFACE1)
							.italics();

						ui.label(text);
					}

					ui.label(
						RichText::new("}")
							.color(colors::SURFACE1)
							.italics(),
					);
				});
			});
		});
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
			ui.vertical_centered(|ui| self.render_status(ui));
		});
	}

	fn render_status(&mut self, ui: &mut Ui) {
		ui.label(match self.running {
			true => RichText::new("Running").color(colors::GREEN),
			false => RichText::new("Stopped").color(colors::RED),
		});
	}
}
// }}}

// vim: fdm=marker fdl=0
