use gpui::{Context, Render, Window, WindowControlArea, div, prelude::*, px, rgb, white};
use gpui_router::NavLink;

pub struct Topbar;

impl Topbar {
	pub fn new() -> Self {
		Self
	}
}

impl Render for Topbar {
	fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		div()
			.flex()
			.flex_shrink_0()
			.px_4()
			.items_center()
			.justify_between()
			.h(px(40.))
			.bg(rgb(0x1a1a1a))
			.text_color(white())
			.child(
				div()
					.flex_grow()
					.window_control_area(WindowControlArea::Drag)
					.child(div().text_sm().child("Hako")),
			)
			.child(
				div()
					.flex()
					.items_center()
					.gap_2()
					.child(
						NavLink::new().to("/tasks").child(
							div()
								.px_2()
								.py_1()
								.rounded_md()
								.text_sm()
								.text_color(rgb(0xaaaaaa))
								.hover(|s| s.bg(rgb(0x333333)).text_color(white()))
								.cursor_pointer()
								.child("⧗"),
						),
					)
					.child(
						div()
							.px_2()
							.py_1()
							.rounded_sm()
							.text_color(rgb(0x888888))
							.hover(|s| s.bg(rgb(0x333333)).text_color(white()))
							.cursor_pointer()
							.child("—")
							.on_mouse_down(
								gpui::MouseButton::Left,
								cx.listener(|_, _, w, _| w.minimize_window()),
							),
					)
					.child(
						div()
							.px_2()
							.py_1()
							.rounded_sm()
							.text_color(rgb(0x888888))
							.hover(|s| s.bg(rgb(0xef4444)).text_color(white()))
							.cursor_pointer()
							.child("✕")
							.on_mouse_down(
								gpui::MouseButton::Left,
								cx.listener(|_, _, w, _| w.remove_window()),
							),
					),
			)
	}
}
