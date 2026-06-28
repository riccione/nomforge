pub mod conflict;
pub mod engine;
pub mod error;
pub mod rules;
pub mod scanner;
pub mod undo;

pub use conflict::{Conflict, ConflictReason, detect_conflicts};
pub use engine::{RenameEngine, RenamePlan, RenameResult};
pub use error::{NomforgeError, Result};
pub use rules::{Case, FileMetadata, RenameContext, RenameRule, SeqPosition};
pub use scanner::{ScanOptions, scan_files};
pub use undo::{
    UndoBatch, UndoEntry, UndoLog, default_undo_log_path, list_batches, load_undo_log, log_renames,
    revert_last, save_undo_log, undo_count,
};
