//! Background job payloads and the pure work functions they run.
//!
//! Scans, planning and renames can take a long time, so the GUI runs them on a
//! worker thread and applies the outcome on the UI thread.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

use nomforge_core::{
    Conflict, RenameEngine, RenamePlan, RenameResult, RenameRule, ScanOptions, apply_plans,
    detect_conflicts, log_renames, revert_last, scan_files_with_warnings,
};

/// Data needed to render a preview after a scan and plan.
pub struct PreviewData {
    pub plans: Vec<RenamePlan>,
    pub conflicts: Vec<Conflict>,
    pub files: Vec<PathBuf>,
    pub skipped: usize,
}

/// Data produced by applying plans.
pub struct ApplyData {
    pub plans: Vec<RenamePlan>,
    pub conflicts: Vec<Conflict>,
    pub results: Vec<RenameResult>,
    pub skipped: usize,
    pub undo_error: Option<String>,
}

/// Result of a completed background job.
pub enum JobOutcome {
    Scan(Result<(Vec<PathBuf>, usize), String>),
    Preview(Result<PreviewData, String>),
    Apply(Result<ApplyData, String>),
    Undo(Result<(usize, usize), String>),
}

/// Scan parameters for a job, used to cache the scanned file list.
#[derive(Debug, Clone, PartialEq)]
pub struct ScanRequest {
    pub dir: PathBuf,
    pub options: ScanOptions,
}

/// Handle to a running background job.
pub struct Job {
    pub rx: Receiver<JobOutcome>,
    /// Keep the status message that was set before this job started.
    pub keep_status: bool,
    /// Scan performed by this job, if any.
    pub scan: Option<ScanRequest>,
}

/// Scan `dir`, returning the files and the number of unreadable entries skipped.
///
/// Errors carry the fully formatted status message.
pub fn scan(dir: &Path, options: &ScanOptions) -> Result<(Vec<PathBuf>, usize), String> {
    scan_files_with_warnings(dir, options)
        .map(|(files, warnings)| (files, warnings.len()))
        .map_err(|e| format!("Scan error: {e}"))
}

/// Scan, plan, and detect conflicts for preview.
pub fn preview(
    dir: &Path,
    options: &ScanOptions,
    rules: Vec<RenameRule>,
) -> Result<PreviewData, String> {
    let (files, skipped) = scan(dir, options)?;
    preview_files(files, rules, skipped)
}

/// Plan files that were already scanned and detect conflicts.
pub fn preview_files(
    files: Vec<PathBuf>,
    rules: Vec<RenameRule>,
    skipped: usize,
) -> Result<PreviewData, String> {
    if files.is_empty() {
        return Ok(PreviewData {
            plans: Vec::new(),
            conflicts: Vec::new(),
            files,
            skipped,
        });
    }

    let engine = RenameEngine::new(rules);
    let plans = engine
        .plan(&files)
        .map_err(|e| format!("Plan error: {e}"))?;
    let conflicts = detect_conflicts(&plans);

    Ok(PreviewData {
        plans,
        conflicts,
        files,
        skipped,
    })
}

/// Scan, plan, and apply the renames unless conflicts are detected.
pub fn apply(
    dir: &Path,
    options: &ScanOptions,
    rules: Vec<RenameRule>,
    history: Option<PathBuf>,
) -> Result<ApplyData, String> {
    let data = preview(dir, options, rules)?;

    if data.plans.is_empty() || !data.conflicts.is_empty() {
        return Ok(ApplyData {
            plans: data.plans,
            conflicts: data.conflicts,
            results: Vec::new(),
            skipped: data.skipped,
            undo_error: None,
        });
    }

    Ok(apply_confirmed(data.plans, history, data.skipped))
}

/// Apply already-confirmed plans and log them for undo.
pub fn apply_confirmed(
    plans: Vec<RenamePlan>,
    history: Option<PathBuf>,
    skipped: usize,
) -> ApplyData {
    let results = apply_plans(&plans);
    let undo_error =
        history.and_then(|path| log_renames(&path, &results).err().map(|e| e.to_string()));

    ApplyData {
        plans,
        conflicts: Vec::new(),
        results,
        skipped,
        undo_error,
    }
}

/// Revert the last undo batch.
pub fn undo(history: &Path) -> Result<(usize, usize), String> {
    if !history.exists() {
        return Err("No undo history found".into());
    }
    revert_last(history).map_err(|e| format!("Undo error: {e}"))
}
