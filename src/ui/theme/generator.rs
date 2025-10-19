use iced::{Theme, theme};

use super::{ThemeMode, palette};

pub fn make_theme(cfg: &super::ThemeConfig, system_mode: theme::Mode) -> Theme {
    let primary =
        palette::parse_hex(&cfg.primary_hex).unwrap_or(iced::Color::from_rgb8(0x58, 0x65, 0xF2));
    let p = match cfg.mode {
        ThemeMode::Light => palette::generate_palette_light(primary),
        ThemeMode::Dark => palette::generate_palette_dark(primary),
        ThemeMode::Auto => match system_mode {
            theme::Mode::Dark => palette::generate_palette_dark(primary),
            theme::Mode::Light | theme::Mode::None => palette::generate_palette_light(primary),
        },
    };

    Theme::custom_with_fn("Application", p, iced::theme::palette::Extended::generate)
}
