pub mod logs;

use lazy_static::lazy_static;
use logs::Log;
use tokio::sync::broadcast;
use tracing_subscriber::fmt::MakeWriter;

lazy_static! {
	/// Global [`broadcast::Sender`] that [`tracing-subscriber`] can send logs to.
	/// We can later subscribe to this sender and display the logs elsewhere.
	pub static ref LOG_CHANNEL: broadcast::Sender<Log> = broadcast::channel(1024).0;
}

pub type LogReceiver = broadcast::Receiver<Log>;

pub struct Logger {
	pub sender: &'static broadcast::Sender<Log>,
}

impl Logger {
	pub fn new() -> Self {
		Self { sender: &LOG_CHANNEL }
	}
}

impl<'w> MakeWriter<'w> for Logger {
	type Writer = Self;

	fn make_writer(&'w self) -> Self::Writer {
		Self::new()
	}
}

impl std::io::Write for Logger {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		use std::io::{Error, ErrorKind};

		let log = Log::try_from(buf).map_err(|err| Error::new(ErrorKind::InvalidData, err))?;

		// If this fails, there are no active receivers. This is totally fine though and we can
		// safely ignore it.
		_ = self.sender.send(log);

		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}
