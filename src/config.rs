use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	#[serde(
		serialize_with = "serde_impls::ser_none_as_empty",
		deserialize_with = "serde_impls::deser_empty_as_none"
	)]
	pub cfg_path: Option<PathBuf>,

	pub gsi_port: u16,
	pub api_url: Url,

	#[serde(
		serialize_with = "serde_impls::ser_none_as_empty",
		deserialize_with = "serde_impls::deser_empty_as_none"
	)]
	pub schnose_api_key: Option<Uuid>,
}

impl Config {
	const DEFAULT_CONTENT: &str = r#"
		cfg_path = ''
		gsi_port = 7878
		api_url = 'https://schnose-api.shuttleapp.rs/api/streamers'
		schnose_api_key = ''
	"#;

	pub fn load() -> Result<Self> {
		let default_location = Self::default_location();
		Self::load_from_file(default_location).context("Failed to load config file.")
	}

	pub fn load_from_file(config_path: PathBuf) -> Result<Self> {
		let config_file = if let Ok(config_file) = std::fs::read_to_string(&config_path) {
			config_file
		} else {
			return Self::create(&config_path);
		};

		toml::from_str(&config_file).context("Failed to parse config file.")
	}

	pub fn default_location() -> PathBuf {
		#[cfg(unix)]
		let mut default_location = {
			let mut xdg_home = std::env::var("XDG_CONFIG_HOME")
				.map(PathBuf::from)
				.unwrap_or_else(|_| {
					let home_dir =
						std::env::var("HOME").expect("Could not locate `$HOME`. What the fuck?");

					let mut home_dir = PathBuf::from(home_dir);
					home_dir.push(".config");
					home_dir
				});

			xdg_home.push("schnose-gsi-client");
			xdg_home
		};

		#[cfg(windows)]
		let mut default_location = {
			let mut local_appdata = std::env::var("LOCALAPPDATA")
				.map(PathBuf::from)
				.unwrap_or_else(|_| {
					std::env::var("USERPROFILE")
						.expect("Could not locate `%USERPROFILE%`. What the fuck?")
						.into()
				});

			local_appdata.push(".schnose-gsi-client");
			local_appdata
		};

		default_location.push("config.toml");
		default_location
	}

	fn create(path: &Path) -> Result<Self> {
		use std::{fs::File, io::Write};

		let config_dir = path.parent().unwrap_or(path);
		std::fs::create_dir_all(config_dir).context("Failed to create config directory.")?;

		let mut config_file = File::create(path).context("Failed to create config file.")?;

		write!(&mut config_file, "{}", Self::DEFAULT_CONTENT.trim())
			.context("Failed to write default config to file.")?;

		Ok(Self::default())
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			cfg_path: None,
			gsi_port: 7878,
			api_url: "https://schnose-api.shuttleapp.rs/api/streamers"
				.parse()
				.unwrap(),
			schnose_api_key: None,
		}
	}
}

mod serde_impls {
	use serde::{Deserialize, Deserializer, Serialize, Serializer};
	use std::str::FromStr;

	pub fn deser_empty_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
	where
		D: Deserializer<'de>,
		T: FromStr,
	{
		Ok(Option::<String>::deserialize(deserializer)?
			.filter(|s| !s.is_empty())
			.and_then(|s| s.parse().ok()))
	}

	pub fn ser_none_as_empty<S, T>(item: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
		T: Serialize,
	{
		match item {
			None => String::new().serialize(serializer),
			Some(item) => item.serialize(serializer),
		}
	}
}
