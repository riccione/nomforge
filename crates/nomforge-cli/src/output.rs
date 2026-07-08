use std::path::Path;

use colored::Colorize;
use nomforge_core::{Conflict, ConflictReason, RenamePlan};

/// Calculate the maximum column width based on filenames, capped at 60.
fn max_filename_width(names: &[&str]) -> usize {
    let max_len = names.iter().map(|n| n.len()).max().unwrap_or(20);
    // Cap at 60 to prevent overly wide output, minimum 20
    max_len.clamp(20, 60)
}

/// Print the preview table for a dry-run.
pub fn print_preview(plans: &[RenamePlan]) {
    if plans.is_empty() {
        println!("{}", "No files to rename.".yellow());
        return;
    }

    // Collect filenames and calculate dynamic widths
    let source_names: Vec<String> = plans
        .iter()
        .map(|p| {
            p.source
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .collect();

    let target_names: Vec<String> = plans
        .iter()
        .map(|p| {
            p.target
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .collect();

    let source_refs: Vec<&str> = source_names.iter().map(|s| s.as_str()).collect();
    let target_refs: Vec<&str> = target_names.iter().map(|s| s.as_str()).collect();

    let source_width = max_filename_width(&source_refs);
    let target_width = max_filename_width(&target_refs);

    println!();
    println!(
        "  {:<source_width$} {} {:<target_width$}",
        "FILE".bold(),
        "->".dimmed(),
        "NEW NAME".bold()
    );
    println!("  {}", "-".repeat(source_width + 3 + target_width).dimmed());

    for (source_name, target_name, plan) in source_names
        .iter()
        .zip(target_names.iter())
        .zip(plans.iter())
        .map(|((s, t), p)| (s, t, p))
    {
        if plan.source == plan.target {
            println!(
                "  {:<source_width$} {} {:<target_width$}",
                source_name.dimmed(),
                "->".dimmed(),
                "(no change)".dimmed()
            );
        } else {
            println!(
                "  {:<source_width$} {} {:<target_width$}",
                source_name,
                "->".green(),
                target_name.green()
            );
        }
    }

    println!();
    let renamed = plans.iter().filter(|p| p.source != p.target).count();
    println!("  {}", format!("Would rename {} file(s).", renamed).cyan());
}

/// Print the result table after applying renames.
pub fn print_results(results: &[nomforge_core::RenameResult]) {
    if results.is_empty() {
        println!("{}", "No results.".yellow());
        return;
    }

    // Collect filenames and calculate dynamic widths
    let source_names: Vec<String> = results
        .iter()
        .map(|r| {
            r.source
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .collect();

    let target_names: Vec<String> = results
        .iter()
        .map(|r| {
            r.target
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .collect();

    let source_refs: Vec<&str> = source_names.iter().map(|s| s.as_str()).collect();
    let target_refs: Vec<&str> = target_names.iter().map(|s| s.as_str()).collect();

    let source_width = max_filename_width(&source_refs);
    let target_width = max_filename_width(&target_refs);

    println!();
    println!(
        "  {:<source_width$} {:<target_width$} {}",
        "FILE".bold(),
        "NEW NAME".bold(),
        "STATUS".bold()
    );
    println!(
        "  {}",
        "-".repeat(source_width + 1 + target_width + 1 + 8).dimmed()
    );

    for (source_name, target_name, result) in source_names
        .iter()
        .zip(target_names.iter())
        .zip(results.iter())
        .map(|((s, t), r)| (s, t, r))
    {
        if result.source == result.target {
            println!(
                "  {:<source_width$} {:<target_width$} {}",
                source_name.dimmed(),
                "(no change)".dimmed(),
                "ok (unchanged)".dimmed()
            );
        } else if result.success {
            println!(
                "  {:<source_width$} {:<target_width$} {}",
                source_name,
                target_name,
                "ok".green().bold()
            );
        } else {
            println!(
                "  {:<source_width$} {:<target_width$} {}",
                source_name,
                target_name,
                "FAILED".red().bold()
            );
            if let Some(err) = &result.error {
                println!(
                    "  {:<source_width$} {:<target_width$}   {}",
                    "",
                    "",
                    err.red()
                );
            }
        }
    }

    println!();
    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();
    let skipped = results
        .iter()
        .filter(|r| r.source == r.target && r.success)
        .count();

    if failed > 0 {
        println!(
            "  {}",
            format!("Renamed {} file(s), {} failed.", succeeded, failed).red()
        );
    } else if skipped > 0 {
        println!(
            "  {}",
            format!(
                "Renamed {} file(s), {} unchanged.",
                succeeded - skipped,
                skipped
            )
            .cyan()
        );
    } else {
        println!(
            "  {}",
            format!("Renamed {} file(s).", succeeded).green().bold()
        );
    }
}

/// Print conflicts.
pub fn print_conflicts(conflicts: &[Conflict]) {
    if conflicts.is_empty() {
        return;
    }

    println!();
    println!("  {}", "CONFLICTS DETECTED:".red().bold());
    for conflict in conflicts {
        let reason = match &conflict.reason {
            ConflictReason::SameTarget(path) => {
                format!("duplicate target: {}", path.display())
            }
            ConflictReason::TargetExists(path) => {
                format!("target already exists: {}", path.display())
            }
        };
        println!("  {} {}", "!".red().bold(), reason.red());
    }
    println!();
}

/// Print scan summary.
pub fn print_scan_summary(dir: &Path, file_count: usize) {
    println!(
        "  Scanning {} ({} file(s) found)...",
        dir.display().to_string().cyan(),
        file_count
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn preview_shows_no_files_message() {
        // Just ensure it doesn't panic
        print_preview(&[]);
    }

    #[test]
    fn preview_with_plans() {
        let plans = vec![
            RenamePlan {
                source: PathBuf::from("/tmp/photo.jpg"),
                target: PathBuf::from("/tmp/renamed.jpg"),
            },
            RenamePlan {
                source: PathBuf::from("/tmp/doc.txt"),
                target: PathBuf::from("/tmp/doc.txt"), // no change
            },
        ];
        print_preview(&plans);
    }

    #[test]
    fn results_shows_no_results_message() {
        print_results(&[]);
    }

    #[test]
    fn results_with_values() {
        let results = vec![
            nomforge_core::RenameResult {
                source: PathBuf::from("/tmp/photo.jpg"),
                target: PathBuf::from("/tmp/renamed.jpg"),
                success: true,
                error: None,
            },
            nomforge_core::RenameResult {
                source: PathBuf::from("/tmp/doc.txt"),
                target: PathBuf::from("/tmp/doc.txt"),
                success: true,
                error: None,
            },
            nomforge_core::RenameResult {
                source: PathBuf::from("/tmp/fail.txt"),
                target: PathBuf::from("/tmp/fail.txt"),
                success: false,
                error: Some("permission denied".into()),
            },
        ];
        print_results(&results);
    }

    #[test]
    fn conflicts_empty() {
        print_conflicts(&[]);
    }

    #[test]
    fn conflicts_with_values() {
        let conflicts = vec![Conflict {
            plan_a: 0,
            plan_b: 1,
            reason: ConflictReason::SameTarget(PathBuf::from("/tmp/result.txt")),
        }];
        print_conflicts(&conflicts);
    }

    #[test]
    fn scan_summary() {
        print_scan_summary(Path::new("/tmp/photos"), 42);
    }
}
