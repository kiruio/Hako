use crate::task::error::{TaskError, TaskResult};
use crate::task::lock::LockKey;
use crate::task::main_task::{ConcurrentTask, TaskContext, TaskType};
use async_trait::async_trait;

pub struct DownloadVersionTask {
	pub version_id: String,
	pub instance_id: String,
}

impl TaskType for DownloadVersionTask {
	const TYPE_NAME: &'static str = "DownloadVersion";
}

#[async_trait]
impl ConcurrentTask for DownloadVersionTask {
	type Output = String;

	async fn execute(&mut self, ctx: &TaskContext) -> TaskResult<Self::Output> {
		if ctx.is_cancelled() {
			return Err(TaskError::Cancelled);
		}

		tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

		if ctx.is_cancelled() {
			return Err(TaskError::Cancelled);
		}

		Ok(format!("Downloaded version {}", self.version_id))
	}

	fn locks(&self) -> Vec<LockKey> {
		vec![
			LockKey::instance("instance", &self.instance_id),
			LockKey::resource("version", &self.version_id),
		]
	}

	fn max_concurrent(&self) -> Option<usize> {
		Some(3)
	}
}
