use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LockKey {
	pub resource_type: String,
	pub resource_id: String,
}

impl LockKey {
	pub fn global(resource_type: &str) -> Self {
		Self {
			resource_type: resource_type.to_string(),
			resource_id: "global".to_string(),
		}
	}

	pub fn instance(resource_type: &str, instance_id: &str) -> Self {
		Self {
			resource_type: resource_type.to_string(),
			resource_id: instance_id.to_string(),
		}
	}

	pub fn resource(resource_type: &str, resource_id: &str) -> Self {
		Self {
			resource_type: resource_type.to_string(),
			resource_id: resource_id.to_string(),
		}
	}
}

pub struct LockManager {
	locks: Arc<RwLock<HashSet<LockKey>>>,
}

impl LockManager {
	pub fn new() -> Self {
		Self {
			locks: Arc::new(RwLock::new(HashSet::new())),
		}
	}

	pub async fn try_acquire(&self, keys: &[LockKey]) -> Result<(), String> {
		let mut locks = self.locks.write().await;

		for key in keys {
			if !locks.insert(key.clone()) {
				return Err(format!("Lock conflict: {:?}", key));
			}
		}

		Ok(())
	}

	pub async fn release(&self, keys: &[LockKey]) {
		let mut locks = self.locks.write().await;
		for key in keys {
			locks.remove(key);
		}
	}
}

impl Default for LockManager {
	fn default() -> Self {
		Self::new()
	}
}
