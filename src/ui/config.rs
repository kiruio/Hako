use iced::{Size, window};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub title: &'static str,
    pub window: window::Settings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "Hako",
            window: window::Settings {
                size: Size::new(1050., 590.),
                min_size: Some(Size::new(1050., 590.)),
                decorations: false,
                transparent: true,
                ..Default::default()
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        Self::default()
    }
}
