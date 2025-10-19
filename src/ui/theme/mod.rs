pub mod generator;
pub mod palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Auto,
    Light,
    Dark,
}

#[derive(Debug, Clone)]
pub struct ThemeConfig {
    pub primary_hex: String,
    pub mode: ThemeMode,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            primary_hex: String::from("#1171D2"),
            mode: ThemeMode::Auto,
        }
    }
}

pub use generator::make_theme;
