use crate::task::error::{TaskError, TaskResult};
use crate::task::handle::{TaskHandle, TaskId, TaskState};
use crate::task::lock::LockManager;
use crate::task::main_task::{BlockingTask, ConcurrentTask, TaskContext};
use crate::task::priority::Priority;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, oneshot, watch};
use uuid::Uuid;

pub struct BlockingExecutor {
	lock_manager: Arc<LockManager>,
	running: Arc<RwLock<HashMap<String, TaskId>>>,
}

impl BlockingExecutor {
	pub fn new(lock_manager: Arc<LockManager>) -> Self {
		Self {
			lock_manager,
			running: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	pub async fn submit<T: BlockingTask>(
		&self,
		mut task: T,
		priority: Priority,
	) -> TaskResult<TaskHandle<T::Output>> {
		let task_type = T::TYPE_NAME;
		let locks = task.locks();

		let has_global_lock = locks.iter().any(|lock| lock.resource_id == "global");
		if has_global_lock {
			let running = self.running.write().await;
			if running.get(task_type).is_some() {
				if !task.queueable() {
					return Err(TaskError::LockConflict(format!(
						"Task {} already running",
						task_type
					)));
				}
			}
		}

		if let Err(e) = self.lock_manager.try_acquire(&locks).await {
			return Err(TaskError::LockConflict(e));
		}

		let task_id = Uuid::new_v4();
		let (cancel_tx, cancel_rx) = watch::channel(false);
		let (result_tx, result_rx) = oneshot::channel();
		let state = Arc::new(RwLock::new(TaskState::Pending));

		let handle = TaskHandle::new(
			task_id,
			priority,
			state.clone(),
			Arc::new(cancel_tx),
			result_rx,
		);

		if has_global_lock {
			let mut running = self.running.write().await;
			running.insert(task_type.to_string(), task_id);
		}

		let lock_manager = self.lock_manager.clone();
		let running_clone = self.running.clone();
		let state_clone = state.clone();
		let task_type_clone = task_type.to_string();

		tokio::spawn(async move {
			*state_clone.write().await = TaskState::Running;

			let ctx = TaskContext::new(cancel_rx);
			let result = task.execute(&ctx).await;

			lock_manager.release(&locks).await;
			if has_global_lock {
				let mut running = running_clone.write().await;
				running.remove(&task_type_clone);
			}

			match &result {
				Ok(_) => *state_clone.write().await = TaskState::Completed,
				Err(_) => *state_clone.write().await = TaskState::Failed,
			}

			let _ = result_tx.send(result);
		});

		Ok(handle)
	}
}

pub struct ConcurrentExecutor {
	lock_manager: Arc<LockManager>,
	semaphore: Option<Arc<tokio::sync::Semaphore>>,
}

impl ConcurrentExecutor {
	pub fn new(lock_manager: Arc<LockManager>, max_concurrent: Option<usize>) -> Self {
		Self {
			lock_manager,
			semaphore: max_concurrent.map(|n| Arc::new(tokio::sync::Semaphore::new(n))),
		}
	}

	pub async fn submit<T: ConcurrentTask>(
		&self,
		task: T,
		priority: Priority,
	) -> TaskResult<TaskHandle<T::Output>> {
		let task_id = Uuid::new_v4();
		let (cancel_tx, cancel_rx) = watch::channel(false);
		let (result_tx, result_rx) = oneshot::channel();
		let state = Arc::new(RwLock::new(TaskState::Pending));

		let handle = TaskHandle::new(
			task_id,
			priority,
			state.clone(),
			Arc::new(cancel_tx),
			result_rx,
		);

		let locks = task.locks();
		if let Err(e) = self.lock_manager.try_acquire(&locks).await {
			return Err(TaskError::LockConflict(e));
		}

		let lock_manager = self.lock_manager.clone();
		let state_clone = state.clone();
		let semaphore = self.semaphore.clone();
		let mut task = task;

		tokio::spawn(async move {
			if let Some(s) = &semaphore {
				let _permit = s.acquire().await.ok();
			}

			*state_clone.write().await = TaskState::Running;

			let ctx = TaskContext::new(cancel_rx);
			let result = task.execute(&ctx).await;

			lock_manager.release(&locks).await;

			match &result {
				Ok(_) => *state_clone.write().await = TaskState::Completed,
				Err(_) => *state_clone.write().await = TaskState::Failed,
			}

			let _ = result_tx.send(result);
		});

		Ok(handle)
	}
}
