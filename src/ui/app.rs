use crate::core::state::AppState;
use crate::ui::components::{navbar::Navbar, topbar::Topbar};
use crate::ui::views::{
	download::DownloadView, home::HomeView, instances::InstancesView, settings::SettingsView,
	tasks::TasksView,
};
use gpui::{Context, Entity, Render, Window, div, prelude::*, rgb};
use gpui_router::{Route, Routes};
use std::sync::Arc;

pub struct HakoApp {
	topbar: Entity<Topbar>,
	navbar: Entity<Navbar>,
	pub state: Arc<AppState>,
}

impl HakoApp {
	pub fn new(ctx: &mut Context<Self>) -> Self {
		let state = Arc::new(AppState::new());
		state.scan_instances();

		Self {
			topbar: ctx.new(|_| Topbar::new(state.clone())),
			navbar: ctx.new(|cx| Navbar::new(state.clone(), cx)),
			state,
		}
	}
}

impl Render for HakoApp {
	fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		let state = self.state.clone();

		div()
			.flex()
			.flex_col()
			.bg(rgb(0x0a0a0a))
			.size_full()
			.child(self.topbar.clone())
			.child(
				div()
					.id("main-content")
					.flex_grow()
					.min_h_0()
					.overflow_y_scroll()
					.child(
						Routes::new()
							.basename("/")
							.child(
								Route::new()
									.index()
									.element(HomeView::render()),
							)
							.child(
								Route::new()
									.path("download")
									.element(DownloadView::render()),
							)
							.child(
								Route::new()
									.path("instances")
									.element(InstancesView::render(state.clone())),
							)
							.child(
								Route::new()
									.path("tasks")
									.element(TasksView::render(state.clone())),
							)
							.child(
								Route::new()
									.path("settings")
									.element(SettingsView::render(state.clone())),
							),
					),
			)
			.child(self.navbar.clone())
	}
}
