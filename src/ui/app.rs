use crate::config::launcher::LauncherConfig;
use crate::game::instance::{GameInstance, InstanceScanner};
use crate::storage::config::ConfigManager;
use anyhow::Result;
use gpui::{Context, Render, Window, div, prelude::*};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

pub struct HakoApp {
	config_manager: ConfigManager,
	launcher_config: Arc<Mutex<LauncherConfig>>,
	instances: Arc<Mutex<Vec<GameInstance>>>,
	cluster_paths: Arc<Mutex<Vec<PathBuf>>>,
}

impl HakoApp {
	pub fn new() -> Result<Self> {
		let config_manager = ConfigManager::new()?;
		let launcher_config = Arc::new(Mutex::new(config_manager.load_launcher_config()?));
		let instances = Arc::new(Mutex::new(Vec::new()));
		let cluster_paths = Arc::new(Mutex::new(Vec::new()));

		if let Some(default_path) = crate::platform::paths::default_minecraft_dir() {
			if default_path.exists() {
				if let Ok(new_instances) = InstanceScanner::scan_cluster(&default_path) {
					let mut instances = instances.lock().unwrap();
					instances.extend(new_instances);
					let mut cluster_paths = cluster_paths.lock().unwrap();
					cluster_paths.push(default_path);
					tracing::info!(
						"Auto-scanned default directory, found {} instances",
						instances.len()
					);
				}
			}
		}

		Ok(Self {
			config_manager,
			launcher_config,
			instances,
			cluster_paths,
		})
	}
}

impl Render for HakoApp {
	fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
		let config = self.launcher_config.lock().unwrap().clone();
		let instances = self.instances.lock().unwrap().clone();
		let cluster_paths = self.cluster_paths.lock().unwrap().clone();

		div().size_full().p_4().bg(gpui::white()).child(
			div()
				.flex()
				.flex_col()
				.gap_4()
				.child(
					div()
						.flex()
						.flex_col()
						.gap_2()
						.child(div().text_lg().child("配置"))
						.child(
							div()
								.p_2()
								.bg(gpui::blue().opacity(0.1))
								.rounded_md()
								.child(format!("主题: {:?}", config.theme))
								.child(format!(
									"窗口大小: {}x{}",
									config.window_width.unwrap_or(900),
									config.window_height.unwrap_or(550)
								)),
						),
				)
				.child(
					div()
						.flex()
						.flex_col()
						.gap_2()
						.child(div().text_lg().child("集群"))
						.child(
							div()
								.p_2()
								.bg(gpui::blue().opacity(0.1))
								.rounded_md()
								.child(
									div()
										.text_sm()
										.child(format!("已添加 {} 个集群", cluster_paths.len())),
								)
								.child(
									div().mt_2().flex().flex_col().gap_1().children(
										cluster_paths.iter().map(|p| {
											div().text_sm().child(p.display().to_string())
										}),
									),
								),
						),
				)
				.child(
					div()
						.flex()
						.flex_col()
						.gap_2()
						.child(div().text_lg().child("实例"))
						.child(
							div()
								.p_2()
								.bg(gpui::blue().opacity(0.1))
								.rounded_md()
								.child(
									div()
										.text_sm()
										.child(format!("扫描到 {} 个实例", instances.len())),
								)
								.child(div().mt_2().flex().flex_col().gap_1().children(
									instances.iter().map(|inst| {
										div().text_sm().child(format!(
											"{} - {}",
											inst.version,
											inst.cluster_path.display()
										))
									}),
								)),
						),
				),
		)
	}
}
