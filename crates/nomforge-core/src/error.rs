use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NomforgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid regex pattern '{pattern}': {reason}")]
    InvalidRegex { pattern: String, reason: String },

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Target path already exists: {0}")]
    TargetAlreadyExists(PathBuf),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("No files matched the given filters")]
    NoFilesFound,

    #[error("Undo log error: {0}")]
    UndoLog(String),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, NomforgeError>;
