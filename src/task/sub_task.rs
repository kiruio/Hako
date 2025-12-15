use crate::task::error::TaskError;
use async_trait::async_trait;

#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
	pub max_retries: u32,
	pub retry_delay_ms: u64,
}

impl Default for RetryPolicy {
	fn default() -> Self {
		Self {
			max_retries: 0,
			retry_delay_ms: 1000,
		}
	}
}

#[async_trait]
pub trait SubTask: Send + Sync {
	async fn execute(&self, ctx: &SubTaskContext) -> Result<(), TaskError>;

	fn condition(&self, _ctx: &SubTaskContext) -> bool {
		true
	}

	fn retry_policy(&self) -> RetryPolicy {
		RetryPolicy::default()
	}
}

pub struct SubTaskContext {
	pub cancelled: tokio::sync::watch::Receiver<bool>,
}

impl SubTaskContext {
	pub fn new(cancelled: tokio::sync::watch::Receiver<bool>) -> Self {
		Self { cancelled }
	}

	pub fn is_cancelled(&self) -> bool {
		*self.cancelled.borrow()
	}
}

pub struct SubTaskChain {
	tasks: Vec<Box<dyn SubTask>>,
}

impl SubTaskChain {
	pub fn new() -> Self {
		Self { tasks: Vec::new() }
	}

	pub fn add<T: SubTask + 'static>(&mut self, task: T) {
		self.tasks.push(Box::new(task));
	}

	pub async fn execute(&self, ctx: &SubTaskContext) -> Result<(), TaskError> {
		for task in &self.tasks {
			if !task.condition(ctx) {
				continue;
			}

			let policy = task.retry_policy();
			let mut last_error = None;

			for attempt in 0..=policy.max_retries {
				if ctx.is_cancelled() {
					return Err(TaskError::Cancelled);
				}

				match task.execute(ctx).await {
					Ok(()) => break,
					Err(e) => {
						last_error = Some(e);
						if attempt < policy.max_retries {
							tokio::time::sleep(std::time::Duration::from_millis(
								policy.retry_delay_ms,
							))
							.await;
						}
					}
				}
			}

			if let Some(e) = last_error {
				return Err(e);
			}
		}

		Ok(())
	}
}

impl Default for SubTaskChain {
	fn default() -> Self {
		Self::new()
	}
}
