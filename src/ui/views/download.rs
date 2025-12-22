use gpui::{div, prelude::*, rgb};

pub fn render_download() -> impl IntoElement {
	div()
		.flex_grow()
		.child("Download View")
		.text_color(rgb(0xffffff))
}
