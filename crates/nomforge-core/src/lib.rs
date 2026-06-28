pub mod conflict;
pub mod engine;
pub mod error;
pub mod rules;
pub mod scanner;

pub use conflict::{Conflict, ConflictReason, detect_conflicts};
pub use engine::{RenameEngine, RenamePlan, RenameResult};
pub use error::{NomforgeError, Result};
pub use rules::{Case, FileMetadata, RenameContext, RenameRule, SeqPosition};
pub use scanner::{ScanOptions, scan_files};
