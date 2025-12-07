#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
	Low = 0,
	Normal = 1,
	High = 2,
	Critical = 3,
}

impl Default for Priority {
	fn default() -> Self {
		Self::Normal
	}
}
