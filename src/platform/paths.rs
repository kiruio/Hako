use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn config_dir() -> Result<PathBuf> {
	dirs::config_dir()
		.context("Failed to get config directory")
		.map(|p| p.join("hako"))
}

pub fn cache_dir() -> Result<PathBuf> {
	std::env::temp_dir()
		.join("hako_cache")
		.canonicalize()
		.or_else(|_| {
			std::fs::create_dir_all(std::env::temp_dir().join("hako_cache"))
				.context("Failed to create cache directory")?;
			Ok(std::env::temp_dir().join("hako_cache"))
		})
}

pub fn default_minecraft_dir() -> Option<PathBuf> {
	dirs::config_dir().map(|p| p.join(".minecraft"))
}
