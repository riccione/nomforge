use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NomforgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid regex pattern '{pattern}': {reason}")]
    InvalidRegex { pattern: String, reason: String },

    #[error("Invalid extension '{ext}': {reason}")]
    InvalidExtension { ext: String, reason: String },

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Target path already exists: {0}")]
    TargetAlreadyExists(PathBuf),

    #[error("Conflict: {0}")]
    Conflict(String),

    /// A semantic undo-log failure (e.g. no history, nothing revertable).
    #[error("Undo log error: {0}")]
    UndoLog(String),

    #[error("failed to preserve corrupt undo log as {}: {source}", backup.display())]
    UndoLogBackup {
        backup: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Walk directory error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

pub type Result<T> = std::result::Result<T, NomforgeError>;
