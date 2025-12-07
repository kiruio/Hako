use crate::task::error::TaskError;
use crate::task::priority::Priority;
use std::sync::Arc;
use tokio::sync::oneshot;
use uuid::Uuid;

pub type TaskId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
	Pending,
	Running,
	Completed,
	Failed,
	Cancelled,
}

pub struct TaskHandle<T> {
	pub id: TaskId,
	pub priority: Priority,
	state: Arc<tokio::sync::RwLock<TaskState>>,
	cancel_tx: Arc<tokio::sync::watch::Sender<bool>>,
	result_rx: oneshot::Receiver<Result<T, TaskError>>,
}

impl<T> TaskHandle<T> {
	pub fn new(
		id: TaskId,
		priority: Priority,
		state: Arc<tokio::sync::RwLock<TaskState>>,
		cancel_tx: Arc<tokio::sync::watch::Sender<bool>>,
		result_rx: oneshot::Receiver<Result<T, TaskError>>,
	) -> Self {
		Self {
			id,
			priority,
			state,
			cancel_tx,
			result_rx,
		}
	}

	pub async fn state(&self) -> TaskState {
		*self.state.read().await
	}

	pub async fn cancel(&self) -> Result<(), TaskError> {
		let mut state = self.state.write().await;
		match *state {
			TaskState::Pending | TaskState::Running => {
				*state = TaskState::Cancelled;
				self.cancel_tx
					.send(true)
					.map_err(|_| TaskError::InvalidState)?;
				Ok(())
			}
			_ => Err(TaskError::InvalidState),
		}
	}

	pub async fn result(self) -> Result<T, TaskError> {
		self.result_rx
			.await
			.map_err(|_| TaskError::Failed("Channel closed".to_string()))?
	}
}
