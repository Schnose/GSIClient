use clap::Parser;
use tracing::Level;

#[derive(Debug, Clone, Parser)]
pub struct Args {
	/// `RUST_LOG` level.
	///
	/// By default only logs from this crate will be emitted. If you want to see logs from
	/// dependencies as well, you can use environment variables instead of this flag. For example,
	/// to enable logs from the `gokz_rs` crate, you can do something like this:
	///
	/// ```sh
	/// RUST_LOG=gokz_rs=INFO,schnose_gsi_client=INFO ./path/to/binary
	/// ```
	///
	/// The environment variable will always take precedence over this flag.
	#[arg(long = "logs")]
	#[clap(default_value_t = Level::INFO)]
	pub log_level: Level,
}

pub fn get() -> Args {
	Args::parse()
}
