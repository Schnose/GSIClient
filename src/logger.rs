use {
	crate::colors,
	eframe::egui::RichText,
	tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
	tracing::error,
};

#[derive(Debug, Clone)]
pub struct LogSender {
	emitter: UnboundedSender<Vec<u8>>,
}

#[derive(Debug)]
pub struct LogReceiver {
	buffer: Vec<u8>,
	receiver: UnboundedReceiver<Vec<u8>>,
}

#[tracing::instrument]
pub fn new() -> (LogSender, LogReceiver) {
	let (sender, receiver) = mpsc::unbounded_channel();
	let sender = LogSender { emitter: sender };
	let receiver = LogReceiver { buffer: Vec::new(), receiver };

	(sender, receiver)
}

#[derive(Clone)]
pub struct Log {
	pub timestamp: RichText,
	pub level: RichText,
	pub message: RichText,
}

impl LogReceiver {
	pub fn current(&mut self) -> Option<Vec<u8>> {
		if let Ok(new_logs) = self.receiver.try_recv() {
			self.buffer.extend(new_logs);
			self.buffer.truncate(usize::MAX / 2);
		}

		if self.buffer.is_empty() {
			None
		} else {
			Some(self.buffer.clone())
		}
	}
}

impl std::io::Write for &LogSender {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		match self.emitter.send(buf.into()) {
			Ok(()) => Ok(buf.len()),
			Err(why) => {
				let message = format!("Failed to send logs: {why:?}");
				error!(message);
				Err(std::io::Error::new(std::io::ErrorKind::Other, message))
			}
		}
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

impl Log {
	pub fn from_slice(logs: &[u8]) -> Vec<Self> {
		String::from_utf8_lossy(logs)
			.lines()
			.filter_map(|line| {
				let json = serde_json::from_str::<serde_json::Value>(line).ok()?;

				let level = json.get("level")?.as_str()?;

				let level = match level {
					"TRACE" => RichText::new("[TRACE]").color(colors::TEAL),
					"DEBUG" => RichText::new("[DEBUG]").color(colors::BLUE),
					"INFO" => RichText::new("[INFO] ").color(colors::GREEN),
					"WARN" => RichText::new("[WARN] ").color(colors::YELLOW),
					"ERROR" => RichText::new("[ERROR]").color(colors::RED),
					level => RichText::new(format!("[{level}]")).color(colors::MAUVE),
				}
				.monospace();

				let (date, time) = json
					.get("timestamp")?
					.as_str()?
					.split_once('T')?;

				let (time, _) = time.split_once('.')?;

				let timestamp = format!("{} {}", date.replace('-', "/"), time);

				let timestamp = RichText::new(timestamp)
					.color(colors::RED)
					.monospace();

				let message = json
					.get("fields")?
					.get("message")?
					.as_str()?;

				let message = RichText::new(message)
					.color(colors::LAVENDER)
					.monospace();

				Some(Log { level, timestamp, message })
			})
			.collect()
	}
}
