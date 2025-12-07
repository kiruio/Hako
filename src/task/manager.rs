use crate::task::error::{TaskError, TaskResult};
use crate::task::executor::{BlockingExecutor, ConcurrentExecutor};
use crate::task::handle::{TaskHandle, TaskId};
use crate::task::lock::LockManager;
use crate::task::main_task::{BlockingTask, ConcurrentTask};
use crate::task::priority::Priority;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TaskManager {
	blocking_executor: Arc<BlockingExecutor>,
	concurrent_executor: Arc<ConcurrentExecutor>,
	lock_manager: Arc<LockManager>,
	tasks: Arc<RwLock<HashMap<TaskId, TaskInfo>>>,
}

struct TaskInfo {
	priority: Priority,
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
		let mut tasks = self.tasks.write().await;
		tasks.insert(handle.id, TaskInfo { priority });
		Ok(handle)
	}

	pub async fn submit_concurrent<T: ConcurrentTask>(
		&self,
		task: T,
		priority: Priority,
	) -> TaskResult<TaskHandle<T::Output>> {
		let handle = self.concurrent_executor.submit(task, priority).await?;
		let mut tasks = self.tasks.write().await;
		tasks.insert(handle.id, TaskInfo { priority });
		Ok(handle)
	}

	pub async fn cancel(&self, task_id: TaskId) -> TaskResult<()> {
		let tasks = self.tasks.read().await;
		if tasks.contains_key(&task_id) {
			drop(tasks);
			Ok(())
		} else {
			Err(TaskError::InvalidState)
		}
	}

	pub async fn boost_priority(&self, task_id: TaskId, priority: Priority) -> TaskResult<()> {
		let mut tasks = self.tasks.write().await;
		if let Some(info) = tasks.get_mut(&task_id) {
			info.priority = priority;
			Ok(())
		} else {
			Err(TaskError::InvalidState)
		}
	}
}

impl Default for TaskManager {
	fn default() -> Self {
		Self::new()
	}
}
