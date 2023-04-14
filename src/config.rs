use {
	color_eyre::{
		eyre::{bail as yeet, Context},
		Result,
	},
	serde::{Deserialize, Deserializer, Serialize, Serializer},
	std::{path::PathBuf, str::FromStr},
	uuid::Uuid,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	#[serde(serialize_with = "ser_none_as_empty")]
	#[serde(deserialize_with = "deser_empty_as_none")]
	pub csgo_cfg_path: Option<PathBuf>,
	pub gsi_port: u16,
	#[serde(serialize_with = "ser_none_as_empty")]
	#[serde(deserialize_with = "deser_empty_as_none")]
	pub schnose_api_key: Option<Uuid>,
}

impl Config {
	#[tracing::instrument]
	pub fn find_path() -> Result<PathBuf> {
		#[cfg(unix)]
		let mut config_dir = if let Ok(directory) = std::env::var("XDG_CONFIG_HOME") {
			PathBuf::from(directory)
		} else {
			let home_dir = std::env::var("HOME").context("Did not find $HOME")?;
			let mut home_dir = PathBuf::from(home_dir);
			home_dir.push(".config");
			home_dir
		};

		#[cfg(windows)]
		let mut config_dir = PathBuf::from(std::env::var("APPDATA").context("Did not find AppData")?);

		if !config_dir.exists() {
			yeet!("Config directory ({}) does not exist!", config_dir.display());
		}

		config_dir.push("schnose_gsi_client");

		// Create config folder if it does not yet exist
		if !config_dir.exists() {
			std::fs::create_dir(&config_dir).context("Failed to create config folder.")?;
		}

		config_dir.push("config.toml");

		// Create default config file if there is no config file yet
		if !&config_dir.exists() {
			use std::io::Write;

			let default_contents = r#"
				csgo_cfg_path = ''
				gsi_port = 8888
				schnose_api_key = ''
			"#
			.trim_start()
			.replace('\t', "");

			let mut config_file = std::fs::File::create(&config_dir)
				.context("Failed to create default config file.")?;

			config_file
				.write(default_contents.as_bytes())
				.context("Failed to write default config contents.")?;
		}

		Ok(config_dir)
	}

	#[tracing::instrument]
	pub fn load() -> Result<Self> {
		let config_dir = Self::find_path()?;

		let config_file =
			std::fs::read_to_string(config_dir).context("Failed to read config file.")?;

		toml::from_str(&config_file).context("Failed to deserialize config file.")
	}
}

fn deser_empty_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
	D: Deserializer<'de>,
	T: FromStr,
{
	Ok(Option::<String>::deserialize(deserializer)?
		.filter(|s| !s.is_empty())
		.and_then(|s| s.parse().ok()))
}

fn ser_none_as_empty<S, T>(item: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
	T: Serialize,
{
	match item {
		None => String::new().serialize(serializer),
		Some(item) => item.serialize(serializer),
	}
}
