use iced::theme::palette::Palette;
use iced::{Color, color};

pub fn parse_hex(hex: &str) -> Option<Color> {
    let s = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
    (s.len() == 6).then_some(())?;
    Some(Color::from_rgb8(
        u8::from_str_radix(&s[0..2], 16).ok()?,
        u8::from_str_radix(&s[2..4], 16).ok()?,
        u8::from_str_radix(&s[4..6], 16).ok()?,
    ))
}

pub fn generate_palette_light(primary: Color) -> Palette {
    Palette {
        background: color!(0xf5f5f5),
        text: Color::BLACK,
        primary,
        success: color!(0x12664f),
        warning: color!(0xb77e33),
        danger: color!(0xc3423f),
    }
}

pub fn generate_palette_dark(primary: Color) -> Palette {
    Palette {
        background: color!(0x1e2021),
        text: Color::from_rgb(0.90, 0.90, 0.90),
        primary,
        success: color!(0x22a073),
        warning: color!(0xffc14e),
        danger: color!(0xff6b6b),
    }
}
