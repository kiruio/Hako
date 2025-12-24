use gpui::{div, prelude::*, rgb};

pub struct DownloadView;

impl DownloadView {
	pub fn render() -> impl IntoElement {
		div().flex().flex_col().flex_grow().p_4().gap_4().child(
			div()
				.text_xl()
				.text_color(rgb(0xffffff))
				.child("Download views"),
		)
	}
}
