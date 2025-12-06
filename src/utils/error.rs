use thiserror::Error;

#[derive(Debug, Error)]
pub enum HakoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Task error: {0}")]
    Task(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Platform error: {0}")]
    Platform(String),
}

pub type Result<T> = std::result::Result<T, HakoError>;
