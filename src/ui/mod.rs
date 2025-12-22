use gpui::{App, TitlebarOptions, WindowBounds, WindowOptions, px};

pub mod app;
pub mod components {
	pub mod navbar;
	pub mod topbar;
}
pub mod views {
	pub mod download;
	pub mod home;
}

const WINDOW_SIZE: gpui::Size<gpui::Pixels> = gpui::size(px(1050.), px(590.));
pub fn build_window_options(cx: &App) -> WindowOptions {
	WindowOptions {
		titlebar: Some(TitlebarOptions {
			title: Some("Hako".into()),
			appears_transparent: true,
			traffic_light_position: None,
		}),
		window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
			None,
			WINDOW_SIZE,
			cx,
		))),
		window_min_size: Some(WINDOW_SIZE),
		..Default::default()
	}
}
