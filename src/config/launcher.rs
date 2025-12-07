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
		other.ram_type.map(|v| self.ram_type = Some(v));
		other.ram_custom.map(|v| self.ram_custom = Some(v));
		other
			.jvm_args
			.as_ref()
			.map(|v| self.jvm_args = Some(v.clone()));
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
		other.theme.as_ref().map(|v| self.theme = Some(v.clone()));
		other.window_width.map(|v| self.window_width = Some(v));
		other.window_height.map(|v| self.window_height = Some(v));
		other.game.as_ref().map(|g| {
			self.game.get_or_insert_with(GameDefaults::default).merge(g);
		});
	}
}
