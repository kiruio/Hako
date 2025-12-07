use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LauncherConfig {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub theme: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub window_width: Option<u32>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub window_height: Option<u32>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub game: Option<GameDefaults>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameDefaults {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ram_type: Option<u8>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub ram_custom: Option<u32>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub jvm_args: Option<String>,
}

impl GameDefaults {
	pub fn merge(&mut self, other: &Self) {
		if other.ram_type.is_some() {
			self.ram_type = other.ram_type;
		}
		if other.ram_custom.is_some() {
			self.ram_custom = other.ram_custom;
		}
		if other.jvm_args.is_some() {
			self.jvm_args = other.jvm_args.clone();
		}
	}
}

impl LauncherConfig {
	pub fn default() -> Self {
		Self {
			theme: Some("default".to_string()),
			window_width: Some(900),
			window_height: Some(550),
			game: Some(GameDefaults {
				ram_type: Some(0),
				ram_custom: Some(15),
				jvm_args: None,
			}),
		}
	}

	pub fn merge(&mut self, other: &Self) {
		if other.theme.is_some() {
			self.theme = other.theme.clone();
		}
		if other.window_width.is_some() {
			self.window_width = other.window_width;
		}
		if other.window_height.is_some() {
			self.window_height = other.window_height;
		}

		if let Some(game) = &other.game {
			self.game
				.get_or_insert_with(GameDefaults::default)
				.merge(game);
		}
	}
}
