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
		if other.ram_type.is_some() {
			self.ram_type = other.ram_type;
		}
		if other.ram_custom.is_some() {
			self.ram_custom = other.ram_custom;
		}
		if other.jvm_args.is_some() {
			self.jvm_args = other.jvm_args.clone();
		}
		if other.game_args.is_some() {
			self.game_args = other.game_args.clone();
		}
	}
}
