use crate::game::instance::GameInstance;
use crate::task::game::download::DownloadProgressState;
use crate::task::handle::TaskId;
use crate::task::manager::TaskManager;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct AppState {
	pub task_manager: Arc<TaskManager>,
	pub instances: Arc<Mutex<Vec<GameInstance>>>,
	pub current_instance: Arc<Mutex<Option<usize>>>,
	pub cluster_path: Arc<Mutex<PathBuf>>,
	pub task_progress: Arc<Mutex<HashMap<TaskId, Arc<Mutex<DownloadProgressState>>>>>,
}

impl AppState {
	pub fn new() -> Self {
		let cluster = crate::core::paths::default_minecraft_dir()
			.unwrap_or_else(|| PathBuf::from(".minecraft"));

		Self {
			task_manager: Arc::new(TaskManager::new()),
			instances: Arc::new(Mutex::new(Vec::new())),
			current_instance: Arc::new(Mutex::new(None)),
			cluster_path: Arc::new(Mutex::new(cluster)),
			task_progress: Arc::new(Mutex::new(HashMap::new())),
		}
	}

	pub fn scan_instances(&self) {
		let path = self.cluster_path.lock().unwrap().clone();
		if let Ok(found) = crate::game::instance::InstanceScanner::scan_cluster(&path) {
			let mut guard = self.instances.lock().unwrap();
			*guard = found;
			tracing::info!("Scanned {} instances from {}", guard.len(), path.display());
		}
	}

	pub fn set_cluster_path(&self, path: PathBuf) {
		*self.cluster_path.lock().unwrap() = path;
		self.scan_instances();
	}

	pub fn select_instance(&self, idx: Option<usize>) {
		*self.current_instance.lock().unwrap() = idx;
	}

	pub fn current_instance(&self) -> Option<GameInstance> {
		let idx = self.current_instance.lock().unwrap().clone()?;
		self.instances.lock().unwrap().get(idx).cloned()
	}

	pub fn register_progress(&self, id: TaskId) -> Arc<Mutex<DownloadProgressState>> {
		let progress = Arc::new(Mutex::new(DownloadProgressState::default()));
		self.task_progress
			.lock()
			.unwrap()
			.insert(id, progress.clone());
		progress
	}

	pub fn cleanup_finished_tasks(&self) {
		self.task_progress
			.lock()
			.unwrap()
			.retain(|_, p| !p.lock().map(|g| g.finished).unwrap_or(true));
	}
}

impl Default for AppState {
	fn default() -> Self {
		Self::new()
	}
}
