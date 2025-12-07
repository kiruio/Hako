use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameConfig {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ram_type: Option<u8>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub ram_custom: Option<u32>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub jvm_args: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub game_args: Option<String>,
}

impl GameConfig {
	pub fn merge(&mut self, other: &Self) {
		other.ram_type.map(|v| self.ram_type = Some(v));
		other.ram_custom.map(|v| self.ram_custom = Some(v));
		other
			.jvm_args
			.as_ref()
			.map(|v| self.jvm_args = Some(v.clone()));
		other
			.game_args
			.as_ref()
			.map(|v| self.game_args = Some(v.clone()));
	}
}
