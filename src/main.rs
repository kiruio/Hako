#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{Font, Settings, Size, window};

use crate::ui::root::Application;

mod core;
mod crash_handler;
mod ui;

fn main() {
    crash_handler::hook();
    env_logger::init();

    let _ = iced::application("Hako", Application::update, Application::view)
        .settings(Settings {
            id: Some("Hako".to_string()),
            fonts: vec![include_bytes!("../resources/fonts/NotoSansSC-Regular.ttf").into()],
            default_font: Font::with_name("Noto Sans SC"),
            antialiasing: true,
            ..Default::default()
        })
        .window(window::Settings {
            size: Size::new(1050., 590.),
            min_size: Some(Size::new(1050., 590.)),
            transparent: true,
            ..Default::default()
        })
        .run();
}
