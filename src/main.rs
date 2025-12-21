use crate::ui::app::HakoApp;
use anyhow::Result;
use gpui::{AppContext, Application, TitlebarOptions, WindowBounds, WindowOptions, px};

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
				window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
					None,
					gpui::size(px(800.), px(600.0)),
					ctx,
				))),
				kind: gpui::WindowKind::Normal,
				window_min_size: Some(gpui::size(px(1050.0), px(590.0))),
				..Default::default()
			},
			|_, c| c.new(|ctx| HakoApp::new(ctx)),
		);
		ctx.activate(true);
	});

	Ok(())
}
