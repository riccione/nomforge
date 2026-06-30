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

    /// The generated filename exceeds the OS limit of 255 bytes.
    ///
    /// On Linux (ext4, XFS, Btrfs), the limit is 255 bytes per filename component.
    /// On macOS (APFS), it's 255 characters (NFD decomposed). On Windows (NTFS),
    /// it's 255 UTF-16 code units. We use 255 bytes as a conservative limit that
    /// works across platforms, since a byte-length check is the safest portable
    /// approach.
    #[error("Filename too long ({length} bytes, max 255): {filename}")]
    FilenameTooLong { filename: String, length: usize },

    #[error("Undo log error: {0}")]
    UndoLog(String),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, NomforgeError>;
