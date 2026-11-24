//! Core library for nomforge: filesystem scanning, rename rules, planning,
//! conflict detection, and the undo log.

#![warn(missing_docs)]

pub mod conflict;
pub mod display;
pub mod engine;
pub mod error;
pub mod rules;
pub mod scanner;
pub mod undo;

pub use conflict::{Conflict, ConflictReason, detect_conflicts};
pub use engine::{RenameEngine, RenamePlan, RenameResult, apply_plans};
pub use error::{NomforgeError, Result};
pub use rules::{Case, RenameContext, RenameRule, SeqPosition};
pub use scanner::{ScanOptions, scan_files, scan_files_with_warnings};
pub use undo::{
    UndoBatch, UndoEntry, UndoLog, default_undo_log_path, list_batches, load_undo_log, log_renames,
    revert_last, save_undo_log, undo_count,
};
