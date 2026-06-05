use thiserror::Error;

pub type Result<T> = std::result::Result<T, SafeSortError>;

#[derive(Error, Debug)]
pub enum SafeSortError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path access denied (skipping): {0}")]
    PermissionDenied(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Scan error: {0}")]
    Scan(String),
}
