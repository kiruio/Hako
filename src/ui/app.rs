use gpui::{Context, Entity, Render, Window, div, prelude::*, rgb};
use gpui_router::{Route, Routes};

use crate::ui::{
	components::{navbar::Navbar, topbar::Topbar},
	views::{download::render_download, home::render_home},
};

pub struct HakoApp {
	topbar: Entity<Topbar>,
	navbar: Entity<Navbar>,
}

impl HakoApp {
	pub fn new(ctx: &mut Context<Self>) -> Self {
		Self {
			topbar: ctx.new(|_| Topbar::new()),
			navbar: ctx.new(|_| Navbar::new()),
		}
	}
}

impl Render for HakoApp {
	fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
		div()
			.flex()
			.flex_col()
			.bg(rgb(0x0a0a0a))
			.size_full()
			.child(self.topbar.clone())
			.child(
				Routes::new()
					.basename("/")
					.child(Route::new().index().element(render_home()))
					.child(Route::new().path("download").element(render_download())),
			)
			.child(self.navbar.clone())
	}
}
