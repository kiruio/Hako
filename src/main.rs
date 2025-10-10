#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::ui::Application;

mod core;
mod crash_handler;
mod ui;

fn main() {
    crash_handler::hook();
    env_logger::init();

    let cfg = ui::config::AppConfig::load();

    iced::application(Application::new, Application::update, Application::view)
        .title(cfg.title)
        .window(cfg.window)
        .style(Application::style)
        .run()
        .expect("Could not run application");
}
