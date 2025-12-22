use gpui::{Context, Render, Window, div, prelude::*, px, rgb, white};
use gpui_router::NavLink;

pub struct Navbar {}

impl Navbar {
	pub fn new() -> Self {
		Self {}
	}
}

impl Render for Navbar {
	fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
		div()
			.flex()
			.px_4()
			.h(px(50.))
			.bg(rgb(0x1a1a1a))
			.text_color(white())
			.child(NavLink::new().to("/").child("Home").cursor_pointer())
			.child(
				NavLink::new()
					.to("/download")
					.child("Download")
					.cursor_pointer(),
			)
	}
}
