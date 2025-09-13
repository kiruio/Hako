#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{Size, window};

use crate::ui::root::Application;

mod core;
mod crash_handler;
mod ui;

fn main() {
    crash_handler::hook();
    env_logger::init();

    iced::application(Application::new, Application::update, Application::view)
        .title("Hako")
        .window(window::Settings {
            size: Size::new(1050., 590.),
            min_size: Some(Size::new(1050., 590.)),
            decorations: false,
            transparent: true,
            ..Default::default()
        })
        .style(Application::style)
        .run()
        .expect("Could not run application");
}
