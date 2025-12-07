use crate::config::game::GameConfig;
use crate::config::launcher::LauncherConfig;
use crate::core::paths;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ConfigManager {
	config_dir: PathBuf,
}

impl ConfigManager {
	pub fn new() -> Result<Self> {
		let config_dir = paths::config_dir()?;
		fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
		Ok(Self { config_dir })
	}

	pub fn load_launcher_config(&self) -> Result<LauncherConfig> {
		let config_path = self.config_dir.join("config.yml");
		if !config_path.exists() {
			return Ok(LauncherConfig::default());
		}
		let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
		let config: LauncherConfig =
			serde_yaml::from_str(&content).context("Failed to parse config file")?;
		let mut default = LauncherConfig::default();
		default.merge(&config);
		Ok(default)
	}

	pub fn save_launcher_config(&self, config: &LauncherConfig) -> Result<()> {
		let config_path = self.config_dir.join("config.yml");
		let yaml = serde_yaml::to_string(config).context("Failed to serialize config")?;
		fs::write(&config_path, yaml).context("Failed to write config file")?;
		Ok(())
	}

	pub fn load_game_config(&self, cluster_path: &Path, version: &str) -> Result<GameConfig> {
		let config_path = cluster_path
			.join("versions")
			.join(version)
			.join("Hako")
			.join("settings.yml");
		if !config_path.exists() {
			return Ok(GameConfig::default());
		}
		let content = fs::read_to_string(&config_path).context("Failed to read game config")?;
		let config: GameConfig =
			serde_yaml::from_str(&content).context("Failed to parse game config")?;
		Ok(config)
	}

	pub fn save_game_config(
		&self,
		cluster_path: &Path,
		version: &str,
		config: &GameConfig,
	) -> Result<()> {
		let config_dir = cluster_path.join("versions").join(version).join("Hako");
		fs::create_dir_all(&config_dir).context("Failed to create game config directory")?;
		let config_path = config_dir.join("settings.yml");
		let yaml = serde_yaml::to_string(config).context("Failed to serialize game config")?;
		fs::write(&config_path, yaml).context("Failed to write game config file")?;
		Ok(())
	}
}
