use gpui::{Context, Render, Window, div, prelude::*};

pub struct HomeView {}

impl HomeView {
	pub fn new() -> Self {
		Self {}
	}
}

impl Render for HomeView {
	fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
		div().flex_grow().child("Home View")
	}
}
