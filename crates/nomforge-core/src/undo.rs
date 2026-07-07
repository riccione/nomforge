use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::engine::RenameResult;
use crate::error::{NomforgeError, Result};

/// A single rename operation in the undo log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoEntry {
    pub source: PathBuf,
    pub target: PathBuf,
}

/// A batch of rename operations that can be undone together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoBatch {
    pub timestamp: DateTime<Utc>,
    pub operations: Vec<UndoEntry>,
}

/// The undo log containing all rename batches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoLog {
    pub batches: Vec<UndoBatch>,
}

impl UndoLog {
    pub fn new() -> Self {
        Self {
            batches: Vec::new(),
        }
    }
}

impl Default for UndoLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the default undo log path (~/.local/share/nomforge/undo_log.json).
pub fn default_undo_log_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("nomforge")
        .join("undo_log.json")
}

/// Load the undo log from disk.
pub fn load_undo_log(path: &Path) -> Result<UndoLog> {
    if !path.exists() {
        return Ok(UndoLog::new());
    }

    let data = fs::read_to_string(path).map_err(|e| NomforgeError::UndoLog(e.to_string()))?;
    let log: UndoLog =
        serde_json::from_str(&data).map_err(|e| NomforgeError::UndoLog(e.to_string()))?;
    Ok(log)
}

/// Save the undo log to disk.
pub fn save_undo_log(path: &Path, log: &UndoLog) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| NomforgeError::UndoLog(e.to_string()))?;
    }

    let data =
        serde_json::to_string_pretty(log).map_err(|e| NomforgeError::UndoLog(e.to_string()))?;
    fs::write(path, data).map_err(|e| NomforgeError::UndoLog(e.to_string()))?;
    Ok(())
}

/// Log a batch of successful rename operations.
pub fn log_renames(path: &Path, results: &[RenameResult]) -> Result<()> {
    let successful: Vec<UndoEntry> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| UndoEntry {
            source: r.source.clone(),
            target: r.target.clone(),
        })
        .collect();

    if successful.is_empty() {
        return Ok(());
    }

    let mut log = load_undo_log(path)?;

    log.batches.push(UndoBatch {
        timestamp: Utc::now(),
        operations: successful,
    });

    save_undo_log(path, &log)
}

/// Revert the last batch of renames.
///
/// Returns the number of files successfully reverted.
///
/// Attempts the rename directly without pre-checking file existence to avoid
/// race conditions where the filesystem state changes between check and rename.
/// If the rename fails (e.g., target doesn't exist, source already exists),
/// that entry is skipped gracefully.
pub fn revert_last(path: &Path) -> Result<usize> {
    let mut log = load_undo_log(path)?;

    let batch = log
        .batches
        .pop()
        .ok_or_else(|| NomforgeError::UndoLog("No undo history found".into()))?;

    let mut reverted = 0;
    for entry in batch.operations.iter().rev() {
        // Attempt rename directly — if it fails, skip this entry.
        // This avoids TOCTOU race conditions between existence checks and rename.
        if fs::rename(&entry.target, &entry.source).is_ok() {
            reverted += 1;
        }
    }

    save_undo_log(path, &log)?;
    Ok(reverted)
}

/// List recent undo batches.
pub fn list_batches(path: &Path, limit: usize) -> Result<Vec<UndoBatch>> {
    let log = load_undo_log(path)?;
    let batches: Vec<UndoBatch> = log.batches.into_iter().rev().take(limit).collect();
    Ok(batches)
}

/// Get the number of undo batches available.
pub fn undo_count(path: &Path) -> Result<usize> {
    let log = load_undo_log(path)?;
    Ok(log.batches.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_undo_path(name: &str) -> PathBuf {
        PathBuf::from(format!("/tmp/nomforge_test_undo_{}", name))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_file(path);
        if let Some(parent) = path.parent()
            && parent != Path::new("/tmp")
        {
            let _ = fs::remove_dir_all(parent);
        }
    }

    fn setup_test_files(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("original.txt"), "content").unwrap();
    }

    #[test]
    fn log_and_list_renames() {
        let path = test_undo_path("log_list");
        cleanup(&path);

        let results = vec![RenameResult {
            source: PathBuf::from("/tmp/a.txt"),
            target: PathBuf::from("/tmp/b.txt"),
            success: true,
            error: None,
        }];

        log_renames(&path, &results).unwrap();
        let batches = list_batches(&path, 10).unwrap();

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].operations.len(), 1);
        assert_eq!(batches[0].operations[0].source, PathBuf::from("/tmp/a.txt"));
        assert_eq!(batches[0].operations[0].target, PathBuf::from("/tmp/b.txt"));

        cleanup(&path);
    }

    #[test]
    fn log_only_successful_renames() {
        let path = test_undo_path("log_successful");
        cleanup(&path);

        let results = vec![
            RenameResult {
                source: PathBuf::from("/tmp/a.txt"),
                target: PathBuf::from("/tmp/b.txt"),
                success: true,
                error: None,
            },
            RenameResult {
                source: PathBuf::from("/tmp/c.txt"),
                target: PathBuf::from("/tmp/d.txt"),
                success: false,
                error: Some("permission denied".into()),
            },
        ];

        log_renames(&path, &results).unwrap();
        let batches = list_batches(&path, 10).unwrap();

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].operations.len(), 1); // Only successful ones

        cleanup(&path);
    }

    #[test]
    fn log_empty_results_is_noop() {
        let path = test_undo_path("log_empty");
        cleanup(&path);

        log_renames(&path, &[]).unwrap();
        let batches = list_batches(&path, 10).unwrap();
        assert!(batches.is_empty());

        cleanup(&path);
    }

    #[test]
    fn multiple_batches() {
        let path = test_undo_path("multi_batch");
        cleanup(&path);

        let results1 = vec![RenameResult {
            source: PathBuf::from("/tmp/a.txt"),
            target: PathBuf::from("/tmp/b.txt"),
            success: true,
            error: None,
        }];
        let results2 = vec![RenameResult {
            source: PathBuf::from("/tmp/c.txt"),
            target: PathBuf::from("/tmp/d.txt"),
            success: true,
            error: None,
        }];

        log_renames(&path, &results1).unwrap();
        log_renames(&path, &results2).unwrap();

        let batches = list_batches(&path, 10).unwrap();
        assert_eq!(batches.len(), 2);

        // Most recent first
        assert_eq!(batches[0].operations[0].source, PathBuf::from("/tmp/c.txt"));
        assert_eq!(batches[1].operations[0].source, PathBuf::from("/tmp/a.txt"));

        cleanup(&path);
    }

    #[test]
    fn undo_count_works() {
        let path = test_undo_path("count");
        cleanup(&path);

        assert_eq!(undo_count(&path).unwrap(), 0);

        let results = vec![RenameResult {
            source: PathBuf::from("/tmp/a.txt"),
            target: PathBuf::from("/tmp/b.txt"),
            success: true,
            error: None,
        }];
        log_renames(&path, &results).unwrap();
        assert_eq!(undo_count(&path).unwrap(), 1);

        cleanup(&path);
    }

    #[test]
    fn revert_last_undos_files() {
        let tmp = PathBuf::from("/tmp/nomforge_undo_revert_test");
        setup_test_files(&tmp);

        let path = test_undo_path("revert");
        cleanup(&path);

        // Create a "successful" rename result
        let results = vec![RenameResult {
            source: tmp.join("original.txt"),
            target: tmp.join("renamed.txt"),
            success: true,
            error: None,
        }];

        // Simulate the rename (since we can't actually rename in tests easily)
        fs::rename(tmp.join("original.txt"), tmp.join("renamed.txt")).unwrap();
        assert!(tmp.join("renamed.txt").exists());
        assert!(!tmp.join("original.txt").exists());

        log_renames(&path, &results).unwrap();
        let reverted = revert_last(&path).unwrap();

        assert_eq!(reverted, 1);
        assert!(tmp.join("original.txt").exists());
        assert!(!tmp.join("renamed.txt").exists());

        cleanup(&path);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn revert_no_history_errors() {
        let path = test_undo_path("no_history");
        cleanup(&path);

        let result = revert_last(&path);
        assert!(result.is_err());

        cleanup(&path);
    }

    #[test]
    fn default_undo_log_path_works() {
        let path = default_undo_log_path();
        assert!(path.to_string_lossy().contains("nomforge"));
        assert!(path.to_string_lossy().ends_with("undo_log.json"));
    }
}
