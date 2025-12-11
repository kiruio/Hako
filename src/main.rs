use crate::ui::app::HakoApp;
use anyhow::Result;
use gpui::{AppContext, Application, WindowOptions};

mod account;
mod config;
mod core;
mod game;
mod net;
mod task;
mod ui;

use core::logger;

fn main() -> Result<()> {
	logger::init();
	let rt = tokio::runtime::Runtime::new()?;
	let _guard = rt.enter();

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
