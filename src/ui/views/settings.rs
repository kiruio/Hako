use crate::core::state::AppState;
use gpui::{div, prelude::*, rgb};
use std::sync::Arc;

pub struct SettingsView;

impl SettingsView {
	pub fn render(state: Arc<AppState>) -> impl IntoElement {
		let cluster_path = state.cluster_path.lock().unwrap().clone();

		div()
			.flex()
			.flex_col()
			.flex_grow()
			.p_4()
			.gap_4()
			.child(div().text_xl().text_color(rgb(0xffffff)).child("设置"))
			.child(Self::render_section(
				"启动器设置",
				div()
					.flex()
					.flex_col()
					.gap_3()
					.child(Self::render_setting_item(
						"游戏目录",
						&cluster_path.display().to_string(),
						"Minecraft 实例存储位置",
					))
					.child(Self::render_setting_item("主题", "深色", "界面主题设置"))
					.child(Self::render_setting_item(
						"语言",
						"简体中文",
						"界面显示语言",
					)),
			))
			.child(Self::render_section(
				"全局游戏默认设置",
				div()
					.flex()
					.flex_col()
					.gap_3()
					.child(Self::render_setting_item(
						"Java 路径",
						"自动检测",
						"游戏使用的 Java 运行时",
					))
					.child(Self::render_setting_item(
						"最大内存",
						"4096 MB",
						"JVM 最大内存分配",
					))
					.child(Self::render_setting_item(
						"窗口大小",
						"854 x 480",
						"游戏窗口默认尺寸",
					))
					.child(Self::render_setting_item(
						"JVM 参数",
						"默认",
						"额外的 JVM 启动参数",
					)),
			))
			.child(Self::render_section(
				"网络设置",
				div()
					.flex()
					.flex_col()
					.gap_3()
					.child(Self::render_setting_item(
						"下载并发数",
						"5",
						"同时下载的文件数量",
					))
					.child(Self::render_setting_item(
						"下载源",
						"官方",
						"游戏文件下载源",
					)),
			))
	}

	fn render_section(title: &str, content: impl IntoElement) -> impl IntoElement {
		div()
			.flex()
			.flex_col()
			.gap_3()
			.p_4()
			.rounded_lg()
			.bg(rgb(0x141414))
			.child(
				div()
					.text_lg()
					.text_color(rgb(0xffffff))
					.mb_2()
					.child(title.to_string()),
			)
			.child(content)
	}

	fn render_setting_item(label: &str, value: &str, desc: &str) -> impl IntoElement {
		div()
			.flex()
			.items_center()
			.justify_between()
			.py_2()
			.border_b_1()
			.border_color(rgb(0x252525))
			.child(
				div()
					.flex()
					.flex_col()
					.gap_1()
					.child(div().text_color(rgb(0xdddddd)).child(label.to_string()))
					.child(
						div()
							.text_xs()
							.text_color(rgb(0x666666))
							.child(desc.to_string()),
					),
			)
			.child(
				div()
					.px_3()
					.py_1()
					.rounded_md()
					.bg(rgb(0x1a1a1a))
					.text_color(rgb(0x888888))
					.text_sm()
					.child(value.to_string()),
			)
	}
}
