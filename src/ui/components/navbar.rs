use gpui::{Context, Render, Window, div, prelude::*, px, rgb};

pub struct Navbar {}

impl Navbar {
	pub fn new() -> Self {
		Self {}
	}
}

impl Render for Navbar {
	fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
		div().flex().h(px(50.)).bg(rgb(0x1a1a1a))
	}
}
