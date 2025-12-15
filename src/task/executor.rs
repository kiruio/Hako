use crate::task::error::{TaskError, TaskResult};
use crate::task::handle::{TaskHandle, TaskId, TaskState};
use crate::task::lock::LockManager;
use crate::task::main_task::{BlockingTask, ConcurrentTask, TaskContext};
use crate::task::priority::Priority;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify, RwLock, Semaphore, oneshot, watch};
use uuid::Uuid;

struct PendingTask {
	task_id: TaskId,
	notify: Arc<Notify>,
}

pub struct BlockingExecutor {
	lock_manager: Arc<LockManager>,
	running: Arc<RwLock<HashMap<String, TaskId>>>,
	queues: Arc<RwLock<HashMap<String, Vec<(Priority, PendingTask)>>>>,
}

impl BlockingExecutor {
	pub fn new(lock_manager: Arc<LockManager>) -> Self {
		Self {
			lock_manager,
			running: Arc::new(RwLock::new(HashMap::new())),
			queues: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	pub async fn boost_priority(
		&self,
		task_type: &str,
		task_id: TaskId,
		new_priority: Priority,
	) -> bool {
		let mut queues = self.queues.write().await;
		if let Some(queue) = queues.get_mut(task_type) {
			if let Some(item) = queue
				.iter_mut()
				.find(|(_, pending)| pending.task_id == task_id)
			{
				item.0 = new_priority;
				queue.sort_by(|a, b| b.0.cmp(&a.0));
				return true;
			}
		}
		false
	}

	pub async fn submit<T: BlockingTask>(
		&self,
		mut task: T,
		priority: Priority,
	) -> TaskResult<TaskHandle<T::Output>> {
		let task_type = T::TYPE_NAME;
		let locks = task.locks();

		let task_id = Uuid::new_v4();
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
				let notify = Arc::new(Notify::new());
				let queues = self.queues.clone();
				let task_type_key = task_type.to_string();
				{
					let mut queues = queues.write().await;
					let queue = queues.entry(task_type_key).or_insert_with(Vec::new);
					queue.push((
						priority,
						PendingTask {
							task_id,
							notify: notify.clone(),
						},
					));
					queue.sort_by(|a, b| b.0.cmp(&a.0));
				}
				drop(running);
				notify.notified().await;
				let running = self.running.write().await;
				if running.get(task_type).is_some() {
					return Err(TaskError::LockConflict(format!(
						"Task {} still running after queue wait",
						task_type
					)));
				}
			}
		}

		if let Err(e) = self.lock_manager.try_acquire(&locks).await {
			return Err(TaskError::LockConflict(e));
		}
		let (cancel_tx, cancel_rx) = watch::channel(false);
		let (result_tx, result_rx) = oneshot::channel();
		let state = Arc::new(RwLock::new(TaskState::Pending));
		let completion = Arc::new(Notify::new());

		let handle_task = TaskHandle::new(
			task_id,
			priority,
			state.clone(),
			Arc::new(cancel_tx),
			completion.clone(),
			result_rx,
		);

		let task_type_str = task_type.to_string();
		if has_global_lock {
			let mut running = self.running.write().await;
			running.insert(task_type_str.clone(), task_id);
		}

		let lock_manager = self.lock_manager.clone();
		let running_clone = self.running.clone();
		let queues_clone = self.queues.clone();
		let state_clone = state.clone();
		let task_type_clone = task_type_str.clone();
		let completion_clone = completion.clone();
		let locks_clone = locks.clone();

		let result_tx_shared = Arc::new(Mutex::new(Some(result_tx)));
		let result_tx_shared_clone = result_tx_shared.clone();

		let handle_spawn = tokio::spawn(async move {
			*state_clone.write().await = TaskState::Running;

			let ctx = TaskContext::new(cancel_rx);
			let result = task.execute(&ctx).await;

			lock_manager.release(&locks_clone).await;
			if has_global_lock {
				let mut running = running_clone.write().await;
				running.remove(&task_type_clone);
				drop(running);
				let mut queues = queues_clone.write().await;
				if let Some(queue) = queues.get_mut(&task_type_clone) {
					if !queue.is_empty() {
						let (_, pending) = queue.remove(0);
						drop(queues);
						pending.notify.notify_one();
					} else {
						queues.remove(&task_type_clone);
					}
				}
			}

			*state_clone.write().await = match result {
				Ok(_) => TaskState::Completed,
				Err(_) => TaskState::Failed,
			};

			if let Some(tx) = result_tx_shared.lock().await.take() {
				let _ = tx.send(result);
			}
			completion_clone.notify_waiters();
		});

		let lock_mgr = self.lock_manager.clone();
		let running = self.running.clone();
		let task_type_str_panic = task_type_str.clone();
		let locks = locks.clone();
		let state = state.clone();
		let completion = completion.clone();

		tokio::spawn(async move {
			if handle_spawn.await.is_err() {
				lock_mgr.release(&locks).await;
				if has_global_lock {
					let mut r = running.write().await;
					r.remove(&task_type_str_panic);
				}
				*state.write().await = TaskState::Failed;
				if let Some(tx) = result_tx_shared_clone.lock().await.take() {
					let _ = tx.send(Err(TaskError::Failed("Task panicked".to_string())));
				}
				completion.notify_waiters();
			}
		});

		Ok(handle_task)
	}
}

pub struct ConcurrentExecutor {
	lock_manager: Arc<LockManager>,
	global_semaphore: Option<Arc<Semaphore>>,
	type_limits: Arc<RwLock<HashMap<String, Arc<TypeGate>>>>,
}

impl ConcurrentExecutor {
	pub fn new(lock_manager: Arc<LockManager>, max_concurrent: Option<usize>) -> Self {
		Self {
			lock_manager,
			global_semaphore: max_concurrent.map(|n| Arc::new(Semaphore::new(n))),
			type_limits: Arc::new(RwLock::new(HashMap::new())),
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
		let completion = Arc::new(Notify::new());

		let handle = TaskHandle::new(
			task_id,
			priority,
			state.clone(),
			Arc::new(cancel_tx),
			completion.clone(),
			result_rx,
		);

		let locks = task.locks();
		if let Err(e) = self.lock_manager.try_acquire(&locks).await {
			return Err(TaskError::LockConflict(e));
		}

		let lock_manager = self.lock_manager.clone();
		let state_clone = state.clone();
		let mut task = task;
		let completion_clone = completion.clone();
		let type_limits = self.type_limits.clone();
		let global_semaphore = self.global_semaphore.clone();
		let max_concurrent = task.max_concurrent();
		let locks_clone = locks.clone();

		tokio::spawn(async move {
			let permit =
				acquire_permit_inner(global_semaphore, type_limits, T::TYPE_NAME, max_concurrent)
					.await;
			if let Err(err) = permit {
				lock_manager.release(&locks_clone).await;
				*state_clone.write().await = TaskState::Failed;
				let _ = result_tx.send(Err(err));
				completion_clone.notify_waiters();
				return;
			}
			let (_global, _type_guard) = permit.unwrap();

			*state_clone.write().await = TaskState::Running;

			let ctx = TaskContext::new(cancel_rx);
			let result = task.execute(&ctx).await;

			lock_manager.release(&locks).await;

			*state_clone.write().await = match result {
				Ok(_) => TaskState::Completed,
				Err(_) => TaskState::Failed,
			};

			let _ = result_tx.send(result);
			completion_clone.notify_waiters();
		});

		Ok(handle)
	}
}

async fn acquire_permit_inner(
	global_semaphore: Option<Arc<Semaphore>>,
	type_limits: Arc<RwLock<HashMap<String, Arc<TypeGate>>>>,
	task_type: &'static str,
	limit: Option<usize>,
) -> TaskResult<(Option<tokio::sync::OwnedSemaphorePermit>, TypePermit)> {
	let global_permit = if let Some(global) = &global_semaphore {
		Some(
			global
				.clone()
				.acquire_owned()
				.await
				.map_err(|_| TaskError::Failed("Semaphore closed".to_string()))?,
		)
	} else {
		None
	};

	let type_gate = {
		let mut map = type_limits.write().await;
		map.entry(task_type.to_string())
			.or_insert_with(|| Arc::new(TypeGate::new(limit)))
			.clone()
	};

	let type_permit = type_gate.acquire(limit).await;
	Ok((global_permit, type_permit))
}

struct TypeGate {
	state: Mutex<TypeGateState>,
	notify: Notify,
}

#[derive(Clone, Copy)]
struct TypeGateState {
	limit: usize,
	in_flight: usize,
}

impl TypeGate {
	fn new(limit: Option<usize>) -> Self {
		Self {
			state: Mutex::new(TypeGateState {
				limit: limit.unwrap_or(usize::MAX),
				in_flight: 0,
			}),
			notify: Notify::new(),
		}
	}

	async fn acquire(self: Arc<Self>, limit: Option<usize>) -> TypePermit {
		let requested = limit.unwrap_or(usize::MAX);
		loop {
			let mut state = self.state.lock().await;
			if requested < state.limit {
				state.limit = requested;
			}
			if state.in_flight < state.limit {
				state.in_flight += 1;
				let gate = self.clone();
				drop(state);
				return TypePermit { gate };
			}
			let notified = self.notify.notified();
			drop(state);
			notified.await;
		}
	}

	async fn release(&self) {
		let mut state = self.state.lock().await;
		state.in_flight = state.in_flight.saturating_sub(1);
		self.notify.notify_one();
	}
}

struct TypePermit {
	gate: Arc<TypeGate>,
}

impl Drop for TypePermit {
	fn drop(&mut self) {
		let gate = self.gate.clone();
		tokio::spawn(async move {
			gate.release().await;
		});
	}
}
