use crate::task::error::{TaskError, TaskResult};
use crate::task::executor::{BlockingExecutor, ConcurrentExecutor};
use crate::task::handle::{TaskHandle, TaskId};
use crate::task::lock::LockManager;
use crate::task::main_task::{BlockingTask, ConcurrentTask};
use crate::task::priority::Priority;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Notify, RwLock, watch};

pub struct TaskManager {
	blocking_executor: Arc<BlockingExecutor>,
	concurrent_executor: Arc<ConcurrentExecutor>,
	lock_manager: Arc<LockManager>,
	tasks: Arc<RwLock<HashMap<TaskId, TaskInfo>>>,
}

struct TaskInfo {
	priority: Priority,
	task_type: Option<String>,
	cancel_tx: Arc<watch::Sender<bool>>,
	completion: Arc<Notify>,
}

impl TaskManager {
	pub fn new() -> Self {
		let lock_manager = Arc::new(LockManager::new());
		Self {
			blocking_executor: Arc::new(BlockingExecutor::new(lock_manager.clone())),
			concurrent_executor: Arc::new(ConcurrentExecutor::new(lock_manager.clone(), Some(5))),
			lock_manager,
			tasks: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	pub async fn submit_blocking<T: BlockingTask>(
		&self,
		task: T,
		priority: Priority,
	) -> TaskResult<TaskHandle<T::Output>> {
		let handle = self.blocking_executor.submit(task, priority).await?;
		self.track_task(&handle, Some(T::TYPE_NAME.to_string()))
			.await;
		Ok(handle)
	}

	pub async fn submit_concurrent<T: ConcurrentTask>(
		&self,
		task: T,
		priority: Priority,
	) -> TaskResult<TaskHandle<T::Output>> {
		let handle = self.concurrent_executor.submit(task, priority).await?;
		self.track_task(&handle, None).await;
		Ok(handle)
	}

	pub async fn cancel(&self, task_id: TaskId) -> TaskResult<()> {
		let tasks = self.tasks.read().await;
		let Some(info) = tasks.get(&task_id) else {
			return Err(TaskError::InvalidState);
		};
		info.cancel_tx
			.send(true)
			.map_err(|_| TaskError::InvalidState)
	}

	pub async fn boost_priority(&self, task_id: TaskId, priority: Priority) -> TaskResult<()> {
		let task_type = {
			let tasks = self.tasks.read().await;
			tasks.get(&task_id).and_then(|info| info.task_type.clone())
		};

		if let Some(task_type) = &task_type {
			if self
				.blocking_executor
				.boost_priority(task_type, task_id, priority)
				.await
			{
				let mut tasks = self.tasks.write().await;
				if let Some(info) = tasks.get_mut(&task_id) {
					info.priority = priority;
				}
				return Ok(());
			}
		}

		let mut tasks = self.tasks.write().await;
		if let Some(info) = tasks.get_mut(&task_id) {
			info.priority = priority;
			Ok(())
		} else {
			Err(TaskError::InvalidState)
		}
	}

	async fn track_task<T>(&self, handle: &TaskHandle<T>, task_type: Option<String>) {
		let mut tasks = self.tasks.write().await;
		let task_id = handle.id;
		let info = TaskInfo {
			priority: handle.priority,
			task_type,
			cancel_tx: handle.cancel_token(),
			completion: handle.completion_notifier(),
		};
		tasks.insert(task_id, info);

		let tasks_clone = self.tasks.clone();
		let completion = handle.completion_notifier();
		tokio::spawn(async move {
			completion.notified().await;
			let mut tasks = tasks_clone.write().await;
			tasks.remove(&task_id);
		});
	}
}

impl Default for TaskManager {
	fn default() -> Self {
		Self::new()
	}
}
