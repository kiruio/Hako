use crate::ui::app::HakoApp;
use anyhow::Result;
use gpui::{
	AppContext, Application, Bounds, Point, Size, TitlebarOptions, WindowBounds, WindowOptions,
	bounds, px, size,
};

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
				titlebar: Some(TitlebarOptions {
					title: Some("Hako".into()),
					appears_transparent: true,
					traffic_light_position: None,
				}),
				window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
					None,
					size(px(800.), px(600.0)),
					ctx,
				))),
				window_min_size: Some(size(px(1050.0), px(590.0))),
				..Default::default()
			},
			|_, c| match HakoApp::new() {
				Ok(app) => c.new(|_| app),
				Err(e) => {
					panic!("Failed to initialize: {}", e);
				}
			},
		);
		ctx.activate(true);
	});

	Ok(())
}
