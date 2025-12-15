use crate::core::paths;
use crate::game::instance::InstanceScanner;
use crate::task::game::download::{DownloadGameTask, DownloadProgressState};
use crate::task::game::start::StartGameTask;
use crate::task::manager::TaskManager;
use crate::task::priority::Priority;
use anyhow::Result;
use gpui::{Context, FocusHandle, Focusable, Render, Window, div, prelude::*};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

pub struct HakoApp {
	task_manager: Arc<TaskManager>,
	instances: Arc<Mutex<Vec<crate::game::instance::GameInstance>>>,
	new_cluster_path: String,
	new_version: String,
	active_field: ActiveField,
	focus_handle: OnceLock<FocusHandle>,
	progress: Arc<Mutex<std::collections::HashMap<String, Arc<Mutex<DownloadProgressState>>>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveField {
	None,
	Cluster,
	Version,
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
			new_cluster_path: paths::default_minecraft_dir()
				.map(|p| p.to_string_lossy().into_owned())
				.unwrap_or_default(),
			new_version: String::new(),
			active_field: ActiveField::None,
			focus_handle: OnceLock::new(),
			progress: Arc::new(Mutex::new(std::collections::HashMap::new())),
		})
	}

	fn cleanup_progress(&mut self) {
		if let Ok(mut m) = self.progress.lock() {
			m.retain(|_, p| p.lock().map(|g| !g.finished).unwrap_or(true));
		}
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

	fn download_instance(&self, instance_idx: usize) {
		let instances = self.instances.clone();
		let task_manager = self.task_manager.clone();
		let progress = self.progress.clone();
		let rt_handle = tokio::runtime::Handle::current();

		rt_handle.spawn(async move {
			let inst = {
				let guard = instances.lock().unwrap();
				guard.get(instance_idx).cloned()
			};

			if let Some(instance) = inst {
				let entry = {
					let mut pg = progress.lock().unwrap();
					let entry = Arc::new(Mutex::new(DownloadProgressState {
						message: "待开始".into(),
						..Default::default()
					}));
					pg.insert(instance.version.clone(), entry.clone());
					entry
				};

				let task = DownloadGameTask {
					cluster_path: instance.cluster_path.clone(),
					version: instance.version.clone(),
					progress: Some(entry),
				};

				match task_manager.submit_concurrent(task, Priority::Normal).await {
					Ok(handle) => {
						tracing::info!("download task submitted: {}", handle.id);
						let tasks_clone = task_manager.clone();
						let h_id = handle.id;
						tokio::spawn(async move {
							let res = handle.result().await;
							match res {
								Ok(_) => tracing::info!("download task ok: {}", h_id),
								Err(e) => tracing::error!("download task failed: {} - {}", h_id, e),
							}
							let _ = tasks_clone;
						});
					}
					Err(e) => {
						tracing::error!("Failed to submit download task: {}", e);
					}
				}
				return;
			}
		});
	}

	fn submit_custom_download(&self, cluster_path: String, version: String) {
		if cluster_path.trim().is_empty() || version.trim().is_empty() {
			tracing::warn!("cluster path or version empty, skip submit");
			return;
		}

		let task_manager = self.task_manager.clone();
		let progress = self.progress.clone();
		let rt_handle = tokio::runtime::Handle::current();

		rt_handle.spawn(async move {
			let entry = {
				let mut pg = progress.lock().unwrap();
				let entry = Arc::new(Mutex::new(DownloadProgressState {
					message: "待开始".into(),
					..Default::default()
				}));
				pg.insert(version.clone(), entry.clone());
				entry
			};

			let task = DownloadGameTask {
				cluster_path: PathBuf::from(cluster_path),
				version: version.clone(),
				progress: Some(entry),
			};

			match task_manager.submit_concurrent(task, Priority::Normal).await {
				Ok(handle) => {
					tracing::info!("download task submitted: {}", handle.id);
					let tasks_clone = task_manager.clone();
					let h_id = handle.id;
					tokio::spawn(async move {
						let res = handle.result().await;
						match res {
							Ok(_) => tracing::info!("download task ok: {}", h_id),
							Err(e) => tracing::error!("download task failed: {} - {}", h_id, e),
						}
						let _ = tasks_clone;
					});
				}
				Err(e) => {
					tracing::error!("Failed to submit download task: {}", e);
				}
			}
		});
	}
}

fn build_editable(
	placeholder: &str,
	value: &str,
	active: bool,
	cx: &mut Context<HakoApp>,
	on_focus: impl Fn(&mut HakoApp) + 'static,
) -> impl IntoElement {
	let placeholder = placeholder.to_string();
	let display = if value.is_empty() {
		div()
			.text_sm()
			.text_color(gpui::black().opacity(0.5))
			.child(placeholder.clone())
	} else {
		div()
			.text_sm()
			.text_color(gpui::black())
			.child(value.to_string())
	};

	let bg = if active {
		gpui::blue().opacity(0.12)
	} else {
		gpui::blue().opacity(0.06)
	};

	let row = div()
		.px_3()
		.py_2()
		.rounded_md()
		.bg(bg)
		.border_1()
		.border_color(if active {
			gpui::blue()
		} else {
			gpui::blue().opacity(0.2)
		})
		.cursor_text()
		.child(display);

	<gpui::Div as gpui::InteractiveElement>::on_mouse_down(
		row,
		gpui::MouseButton::Left,
		cx.listener(
			move |app: &mut HakoApp, _event, window: &mut Window, cx: &mut Context<HakoApp>| {
				on_focus(app);
				cx.focus_self(window);
				cx.notify();
			},
		),
	)
}

fn handle_key_edit(target: &mut String, ev: &gpui::KeyDownEvent) {
	let key = ev.keystroke.key.to_lowercase();
	if key == "backspace" {
		target.pop();
		return;
	}
	if key == "enter" {
		return;
	}
	if let Some(c) = ev.keystroke.key_char.as_ref() {
		if c.chars().count() == 1 {
			target.push_str(c);
			return;
		}
	}
	if key.len() == 1 {
		target.push_str(&key);
	}
}

impl Focusable for HakoApp {
	fn focus_handle(&self, cx: &gpui::App) -> FocusHandle {
		self.focus_handle
			.get_or_init(|| cx.focus_handle().tab_stop(true).tab_index(0))
			.clone()
	}
}

impl Render for HakoApp {
	fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		let instances = self.instances.lock().unwrap().clone();
		let form_cluster = self.new_cluster_path.clone();
		let form_version = self.new_version.clone();
		let active = self.active_field;
		let progresses = self.progress.lock().unwrap().clone();

		div()
			.size_full()
			.p_4()
			.bg(gpui::white())
			.track_focus(&self.focus_handle(cx))
			.on_key_down(cx.listener(
				move |app: &mut HakoApp,
				      ev: &gpui::KeyDownEvent,
				      _window: &mut Window,
				      cx: &mut Context<HakoApp>| {
					match app.active_field {
						ActiveField::Cluster => handle_key_edit(&mut app.new_cluster_path, ev),
						ActiveField::Version => handle_key_edit(&mut app.new_version, ev),
						ActiveField::None => {}
					}
					cx.notify();
				},
			))
			.child(
				div()
					.flex()
					.flex_col()
					.gap_4()
					.child(div().text_xl().child("Minecraft 实例"))
					.child(
						div()
							.flex()
							.flex_col()
							.gap_2()
							.p_3()
							.rounded_md()
							.bg(gpui::blue().opacity(0.04))
							.child(div().text_lg().child("提交新下载任务"))
							.child(
								div()
									.flex()
									.flex_col()
									.gap_2()
									.child(
										div()
											.text_sm()
											.text_color(gpui::black().opacity(0.65))
											.child("任务进度"),
									)
									.children(progresses.iter().map(|(ver, p)| {
										let snapshot = p.lock().ok().map(|g| g.clone());
										let (msg, downloaded, total, speed) = snapshot
											.map(|s| {
												(s.message, s.downloaded, s.total, s.speed_bps)
											})
											.unwrap_or_else(|| ("不可用".into(), 0, None, 0.0));
										let total_text = total
											.map(|t| format!("{}", t))
											.unwrap_or_else(|| "?".into());
										let speed = if speed > 0.0 {
											format!("{:.1} KB/s", speed / 1024.0)
										} else {
											"-".into()
										};
										let progress_text = format!(
											"[{}] {} | {}/{} | {}",
											ver, msg, downloaded, total_text, speed
										);
										div()
											.px_2()
											.py_1()
											.bg(gpui::blue().opacity(0.04))
											.rounded_md()
											.child(progress_text)
									}))
									.child(build_editable(
										"实例目录，例如 C:\\\\Users\\\\...\\\\.minecraft",
										&form_cluster,
										active == ActiveField::Cluster,
										cx,
										|app| {
											app.active_field = ActiveField::Cluster;
										},
									))
									.child(build_editable(
										"版本号，例如 1.20.1",
										&form_version,
										active == ActiveField::Version,
										cx,
										|app| {
											app.active_field = ActiveField::Version;
										},
									))
									.child({
										let btn = div()
											.px_3()
											.py_2()
											.rounded_md()
											.bg(gpui::blue().opacity(0.18))
											.child("提交下载")
											.cursor_pointer();
										<gpui::Div as gpui::InteractiveElement>::on_mouse_down(
											btn,
											gpui::MouseButton::Left,
											cx.listener(
												move |app: &mut HakoApp,
												      _event,
												      _window: &mut Window,
												      _cx: &mut Context<HakoApp>| {
													let path = app.new_cluster_path.clone();
													let ver = app.new_version.clone();
													app.submit_custom_download(path, ver);
												},
											),
										)
									}),
							),
					)
					.child(div().text_xl().child("Minecraft 实例列表"))
					.child(div().flex().flex_col().gap_2().children(
						instances.iter().enumerate().map(|(idx, inst)| {
							let info = format!(
								"版本: {}, 路径: {}",
								inst.version,
								inst.cluster_path.display()
							);
							let start_btn = {
								let btn = div()
									.px_2()
									.py_1()
									.rounded_md()
									.bg(gpui::green().opacity(0.12))
									.child("启动")
									.cursor_pointer();
								<gpui::Div as gpui::InteractiveElement>::on_mouse_down(
									btn,
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
							};

							let download_btn = {
								let btn = div()
									.px_2()
									.py_1()
									.rounded_md()
									.bg(gpui::blue().opacity(0.12))
									.child("补全/下载")
									.cursor_pointer();
								<gpui::Div as gpui::InteractiveElement>::on_mouse_down(
									btn,
									gpui::MouseButton::Left,
									cx.listener(
										move |app: &mut HakoApp,
										      _event,
										      _window: &mut Window,
										      _cx: &mut Context<HakoApp>| {
											app.download_instance(idx);
										},
									),
								)
							};

							div()
								.flex()
								.items_center()
								.justify_between()
								.gap_3()
								.p_2()
								.rounded_md()
								.bg(gpui::blue().opacity(0.05))
								.child(div().child(info))
								.child(
									div()
										.flex()
										.gap_2()
										.items_center()
										.children(vec![start_btn, download_btn]),
								)
						}),
					)),
			)
	}
}
