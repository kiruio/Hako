use gpui::{
	Context, InteractiveElement, Render, Window, WindowControlArea, div, prelude::*, px, rgb, white,
};

pub struct Topbar {
	title: String,
}

impl Topbar {
	pub fn new() -> Self {
		Self {
			title: "Hako".to_string(),
		}
	}
}

impl Render for Topbar {
	fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		div()
			.window_control_area(WindowControlArea::Drag)
			.flex()
			.px_4()
			.items_center()
			.justify_between()
			.h(px(40.))
			.bg(rgb(0x2a2a2a))
			.text_color(white())
			.child(div().child(self.title.clone()))
			.child(
				div()
					.flex()
					.gap_2()
					.child(div().cursor_pointer().child("-").on_mouse_down(
						gpui::MouseButton::Left,
						cx.listener(|_, _, window, _| window.minimize_window()),
					))
					.child(div().cursor_pointer().child("X").on_mouse_down(
						gpui::MouseButton::Left,
						cx.listener(|_, _, window, _| window.remove_window()),
					)),
			)
	}
}
