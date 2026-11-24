//! Error types shared across the core crate.

use std::path::PathBuf;

use thiserror::Error;

/// Errors returned by nomforge operations.
#[derive(Debug, Error)]
pub enum NomforgeError {
    /// Underlying filesystem error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// A regex pattern could not be compiled.
    #[error("Invalid regex pattern '{pattern}': {reason}")]
    InvalidRegex {
        /// The offending pattern.
        pattern: String,
        /// Why compilation failed.
        reason: String,
    },

    /// A file extension was rejected by validation.
    #[error("Invalid extension '{ext}': {reason}")]
    InvalidExtension {
        /// The offending extension.
        ext: String,
        /// Why it was rejected.
        reason: String,
    },

    /// A path that should exist was not found.
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// A rename target already exists on disk.
    #[error("Target path already exists: {0}")]
    TargetAlreadyExists(PathBuf),

    /// A validation or planning conflict, such as an invalid rule value.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// A semantic undo-log failure (e.g. no history, nothing revertable).
    #[error("Undo log error: {0}")]
    UndoLog(String),

    /// A corrupt undo log could not be moved aside.
    #[error("failed to preserve corrupt undo log as {}: {source}", backup.display())]
    UndoLogBackup {
        /// Path the corrupt log was being moved to.
        backup: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: std::io::Error,
    },

    /// JSON (de)serialization failed.
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// Directory traversal failed.
    #[error("Walk directory error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

/// Convenience result type for nomforge operations.
pub type Result<T> = std::result::Result<T, NomforgeError>;
