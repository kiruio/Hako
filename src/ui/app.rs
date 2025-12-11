use crate::core::paths;
use crate::game::instance::InstanceScanner;
use crate::task::game::start::StartGameTask;
use crate::task::manager::TaskManager;
use crate::task::priority::Priority;
use anyhow::Result;
use gpui::{Context, Render, Window, div, prelude::*};
use std::sync::{Arc, Mutex};

pub struct HakoApp {
	task_manager: Arc<TaskManager>,
	instances: Arc<Mutex<Vec<crate::game::instance::GameInstance>>>,
}

impl HakoApp {
	pub fn new() -> Result<Self> {
		let instances = Arc::new(Mutex::new(Vec::new()));
		if let Some(default_path) = paths::default_minecraft_dir() {
			if let Ok(found) = InstanceScanner::scan_cluster(&default_path) {
				let mut guard = instances.lock().unwrap();
				*guard = found;
				tracing::info!(
					"Auto-scanned default directory, found {} instances",
					guard.len()
				);
			}
		}

		Ok(Self {
			task_manager: Arc::new(TaskManager::new()),
			instances,
		})
	}

	fn start_instance(&self, instance_idx: usize) {
		let instances = self.instances.clone();
		let task_manager = self.task_manager.clone();
		let rt_handle = tokio::runtime::Handle::current();

		rt_handle.spawn(async move {
			let inst = {
				let guard = instances.lock().unwrap();
				guard.get(instance_idx).cloned()
			};

			if let Some(instance) = inst {
				let task = StartGameTask {
					instance,
					java_path: None,
					jvm_args: Vec::new(),
					game_args: Vec::new(),
				};

				match task_manager.submit_blocking(task, Priority::Normal).await {
					Ok(handle) => {
						tracing::info!("start task submitted: {}", handle.id);
						let tasks_clone = task_manager.clone();
						let h_id = handle.id;
						tokio::spawn(async move {
							let res = handle.result().await;
							match res {
								Ok(_) => tracing::info!("start task ok: {}", h_id),
								Err(e) => tracing::error!("start task failed: {} - {}", h_id, e),
							}
							let _ = tasks_clone;
						});
					}
					Err(e) => {
						tracing::error!("Failed to submit start task: {}", e);
					}
				}
			}
		});
	}
}

impl Render for HakoApp {
	fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		let instances = self.instances.lock().unwrap().clone();

		div().size_full().p_4().bg(gpui::white()).child(
			div()
				.flex()
				.flex_col()
				.gap_3()
				.child(div().text_xl().child("Minecraft 实例列表"))
				.child(
					div()
						.flex()
						.flex_col()
						.gap_2()
						.children(instances.iter().enumerate().map(|(idx, inst)| {
							let info = format!(
								"版本: {}, 路径: {}",
								inst.version,
								inst.cluster_path.display()
							);
							let row = div()
								.flex()
								.items_center()
								.justify_between()
								.gap_2()
								.p_2()
								.rounded_md()
								.bg(gpui::blue().opacity(0.05))
								.child(div().child(info))
								.cursor_pointer();
							<gpui::Div as gpui::InteractiveElement>::on_mouse_down(
								row,
								gpui::MouseButton::Left,
								cx.listener(
									move |app: &mut HakoApp,
									      _event,
									      _window: &mut Window,
									      _cx: &mut Context<HakoApp>| {
										app.start_instance(idx);
									},
								),
							)
						})),
				),
		)
	}
}
