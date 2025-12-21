use gpui::{Context, Entity, Render, Window, div, prelude::*, rgb};

use crate::ui::components::{navbar::Navbar, topbar::Topbar};
use crate::ui::views::home::HomeView;

pub struct HakoApp {
	topbar: Entity<Topbar>,
	navbar: Entity<Navbar>,
	current_view: Entity<HomeView>,
}

impl HakoApp {
	pub fn new(ctx: &mut Context<Self>) -> Self {
		Self {
			topbar: ctx.new(|_| Topbar::new()),
			navbar: ctx.new(|_| Navbar::new()),
			current_view: ctx.new(|_| HomeView::new()),
		}
	}
}

impl Render for HakoApp {
	fn render(&mut self, _window: &mut Window, _ctx: &mut Context<Self>) -> impl IntoElement {
		div()
			.flex()
			.flex_col()
			.bg(rgb(0x0a0a0a))
			.size_full()
			.child(self.topbar.clone())
			.child(self.current_view.clone())
			.child(self.navbar.clone())
	}
}
