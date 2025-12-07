use crate::ui::app::HakoApp;
use anyhow::Result;
use gpui::{AppContext, Application, WindowOptions};

mod config;
mod game;
mod platform;
mod storage;
mod ui;
mod utils;

use utils::logger;

fn main() -> Result<()> {
	logger::init();
	tracing::info!("Hako starting...");

	Application::new().run(|ctx| {
		let _ = ctx.open_window(
			WindowOptions {
				..Default::default()
			},
			|_, c| match HakoApp::new() {
				Ok(app) => c.new(|_| app),
				Err(e) => {
					panic!("Failed to initialize: {}", e);
				}
			},
		);
	});

	Ok(())
}
