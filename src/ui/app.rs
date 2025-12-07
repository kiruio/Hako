use crate::config::launcher::LauncherConfig;
use crate::config::manager::ConfigManager;
use crate::game::instance::{GameInstance, InstanceScanner};
use crate::task::game::download::DownloadVersionTask;
use crate::task::handle::{TaskId, TaskState};
use crate::task::manager::TaskManager;
use crate::task::priority::Priority;
use anyhow::Result;
use gpui::{Context, Render, Window, div, prelude::*};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
struct TaskInfo {
	id: TaskId,
	name: String,
	state: TaskState,
	priority: Priority,
}

pub struct HakoApp {
	launcher_config: Arc<Mutex<LauncherConfig>>,
	instances: Arc<Mutex<Vec<GameInstance>>>,
	cluster_paths: Arc<Mutex<Vec<PathBuf>>>,
	task_manager: Arc<TaskManager>,
	tasks: Arc<Mutex<Vec<TaskInfo>>>,
}

impl HakoApp {
	pub fn new() -> Result<Self> {
		let config_manager = ConfigManager::new()?;
		let launcher_config = Arc::new(Mutex::new(config_manager.load_launcher_config()?));
		let instances = Arc::new(Mutex::new(Vec::new()));
		let cluster_paths = Arc::new(Mutex::new(Vec::new()));
		let task_manager = Arc::new(TaskManager::new());
		let tasks = Arc::new(Mutex::new(Vec::new()));

		if let Some(default_path) = crate::core::paths::default_minecraft_dir() {
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

		let app = Self {
			launcher_config,
			instances,
			cluster_paths,
			task_manager,
			tasks,
		};

		app.init_tasks();

		Ok(app)
	}
}

impl HakoApp {
	fn submit_task_internal(&self, version_id: String, delay_ms: u64) {
		let task_manager = self.task_manager.clone();
		let tasks = self.tasks.clone();

		let rt_handle = tokio::runtime::Handle::current();
		let rt_handle_clone = rt_handle.clone();
		rt_handle.spawn(async move {
			if delay_ms > 0 {
				tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
			}

			let task = DownloadVersionTask {
				version_id: version_id.clone(),
				instance_id: "demo".to_string(),
			};

			if let Ok(task_handle) = task_manager.submit_concurrent(task, Priority::Normal).await {
				let task_id = task_handle.id;
				let state = task_handle.state().await;
				let mut tasks_guard = tasks.lock().unwrap();

				tasks_guard.push(TaskInfo {
					id: task_id,
					name: format!("Download {}", version_id),
					state,
					priority: task_handle.priority,
				});

				tracing::info!("Task submitted: {}", version_id);
				drop(tasks_guard);

				let tasks_clone = tasks.clone();
				rt_handle_clone.spawn(async move {
					let result = task_handle.result().await;
					tracing::info!("Task completed: {:?}", result);

					let mut tasks_guard = tasks_clone.lock().unwrap();
					if let Some(task_info) = tasks_guard.iter_mut().find(|t| t.id == task_id) {
						task_info.state = match &result {
							Ok(_) => TaskState::Completed,
							Err(_) => TaskState::Failed,
						};
					}
				});
			}
		});
	}

	fn init_tasks(&self) {
		for i in 0..3 {
			let version_id = format!("1.20.{}", i + 1);
			self.submit_task_internal(version_id, i * 200);
		}
	}
}

impl Render for HakoApp {
	fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
		let config = self.launcher_config.lock().unwrap().clone();
		let instances = self.instances.lock().unwrap().clone();
		let cluster_paths = self.cluster_paths.lock().unwrap().clone();
		let tasks = self.tasks.lock().unwrap().clone();

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
				)
				.child(
					div()
						.flex()
						.flex_col()
						.gap_2()
						.child(div().text_lg().child("任务演示"))
						.child(
							div()
								.p_2()
								.bg(gpui::blue().opacity(0.1))
								.rounded_md()
								.child(
									div()
										.flex()
										.gap_2()
										.items_center()
										.child(
											div()
												.px_4()
												.py_2()
												.bg(gpui::blue())
												.rounded_md()
												.text_color(gpui::white())
												.child(
													div()
														.text_sm()
														.child("任务演示 - 自动提交了3个示例任务"),
												),
										)
										.child(
											div()
												.text_sm()
												.child(format!("总任务数: {}", tasks.len())),
										),
								)
								.child(div().mt_2().flex().flex_col().gap_1().children(
									tasks.iter().map(|task| {
										let state_str = match task.state {
											TaskState::Pending => "等待中",
											TaskState::Running => "运行中",
											TaskState::Completed => "已完成",
											TaskState::Failed => "失败",
											TaskState::Cancelled => "已取消",
										};
										let priority_str = match task.priority {
											Priority::Low => "低",
											Priority::Normal => "普通",
											Priority::High => "高",
											Priority::Critical => "紧急",
										};
										div()
											.p_2()
											.bg(gpui::blue().opacity(0.05))
											.rounded_md()
											.flex()
											.flex_col()
											.gap_1()
											.child(div().text_sm().child(task.name.clone()))
											.child(
												div()
													.flex()
													.gap_2()
													.text_xs()
													.child(
														div().child(format!("状态: {}", state_str)),
													)
													.child(
														div().child(format!(
															"优先级: {}",
															priority_str
														)),
													)
													.child(div().child(format!(
														"ID: {}",
														task.id.as_simple()
													))),
											)
									}),
								)),
						),
				),
		)
	}
}
