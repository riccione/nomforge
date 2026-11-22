use std::borrow::Cow;
use std::path::Path;

use colored::Colorize;
use nomforge_core::{Conflict, ConflictReason, RenamePlan, RenameResult};
use unicode_width::UnicodeWidthStr;

/// File name for display, borrowing when the path is valid UTF-8.
fn display_name(path: &Path) -> Cow<'_, str> {
    path.file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default()
}

/// Display width in terminal columns (CJK and emoji take two columns).
fn display_width(text: &str) -> usize {
    text.width()
}

/// Pad `text` with spaces up to `width` display columns.
///
/// Pad before colorizing: ANSI escape sequences emitted by `colored` would
/// otherwise be counted towards the formatted width.
fn pad(text: &str, width: usize) -> String {
    let padding = width.saturating_sub(display_width(text));
    let mut padded = String::with_capacity(text.len() + padding);
    padded.push_str(text);
    padded.extend(std::iter::repeat_n(' ', padding));
    padded
}

/// Maximum column width in display columns, capped at 60, minimum 20.
fn max_width(names: &[Cow<'_, str>]) -> usize {
    names
        .iter()
        .map(|name| display_width(name))
        .max()
        .unwrap_or(20)
        .clamp(20, 60)
}

/// Collect source and target display names for table rows.
fn columns<'a, T>(
    items: &'a [T],
    source: impl Fn(&'a T) -> &'a Path,
    target: impl Fn(&'a T) -> &'a Path,
) -> (Vec<Cow<'a, str>>, Vec<Cow<'a, str>>) {
    let sources = items
        .iter()
        .map(|item| display_name(source(item)))
        .collect();
    let targets = items
        .iter()
        .map(|item| display_name(target(item)))
        .collect();
    (sources, targets)
}

/// Counts shown in the results summary.
struct Summary {
    renamed: usize,
    failed: usize,
    skipped: usize,
}

/// Summarize results, never counting unchanged entries as renamed.
fn summarize(results: &[RenameResult]) -> Summary {
    let failed = results.iter().filter(|r| !r.success).count();
    let skipped = results
        .iter()
        .filter(|r| r.success && r.source == r.target)
        .count();
    let succeeded = results.len() - failed;

    Summary {
        renamed: succeeded - skipped,
        failed,
        skipped,
    }
}

/// Print the preview table for a dry-run.
pub fn print_preview(plans: &[RenamePlan]) {
    if plans.is_empty() {
        println!("{}", "No files to rename.".yellow());
        return;
    }

    let (sources, targets) = columns(plans, |plan| &plan.source, |plan| &plan.target);
    let source_width = max_width(&sources);
    let target_width = max_width(&targets);

    println!();
    println!(
        "  {} {} {}",
        pad("FILE", source_width).bold(),
        "->".dimmed(),
        pad("NEW NAME", target_width).bold()
    );
    println!(
        "  {}",
        "-".repeat(source_width + 1 + 2 + 1 + target_width).dimmed()
    );

    for ((source, target), plan) in sources.iter().zip(targets.iter()).zip(plans) {
        if plan.source == plan.target {
            println!(
                "  {} {} {}",
                pad(source, source_width).dimmed(),
                "->".dimmed(),
                pad("(no change)", target_width).dimmed()
            );
        } else {
            println!(
                "  {} {} {}",
                pad(source, source_width),
                "->".green(),
                pad(target, target_width).green()
            );
        }
    }

    println!();
    let renamed = plans.iter().filter(|p| p.source != p.target).count();
    println!("  {}", format!("Would rename {} file(s).", renamed).cyan());
}

/// Print the result table after applying renames.
pub fn print_results(results: &[RenameResult]) {
    if results.is_empty() {
        println!("{}", "No results.".yellow());
        return;
    }

    let (sources, targets) = columns(results, |result| &result.source, |result| &result.target);
    let source_width = max_width(&sources);
    let target_width = max_width(&targets);
    let status_width = display_width("STATUS");

    println!();
    println!(
        "  {} {} {}",
        pad("FILE", source_width).bold(),
        pad("NEW NAME", target_width).bold(),
        "STATUS".bold()
    );
    println!(
        "  {}",
        "-".repeat(source_width + 1 + target_width + 1 + status_width)
            .dimmed()
    );

    for ((source, target), result) in sources.iter().zip(targets.iter()).zip(results) {
        if result.source == result.target {
            println!(
                "  {} {} {}",
                pad(source, source_width).dimmed(),
                pad("(no change)", target_width).dimmed(),
                "ok (unchanged)".dimmed()
            );
        } else if result.success {
            println!(
                "  {} {} {}",
                pad(source, source_width),
                pad(target, target_width),
                "ok".green().bold()
            );
        } else {
            println!(
                "  {} {} {}",
                pad(source, source_width),
                pad(target, target_width),
                "FAILED".red().bold()
            );
            if let Some(err) = &result.error {
                println!(
                    "  {} {}   {}",
                    " ".repeat(source_width),
                    " ".repeat(target_width),
                    err.red()
                );
            }
        }
    }

    println!();
    let Summary {
        renamed,
        failed,
        skipped,
    } = summarize(results);

    if failed > 0 {
        println!(
            "  {}",
            format!("Renamed {renamed} file(s), {failed} failed.").red()
        );
    } else if skipped > 0 {
        println!(
            "  {}",
            format!("Renamed {renamed} file(s), {skipped} unchanged.").cyan()
        );
    } else {
        println!("  {}", format!("Renamed {renamed} file(s).").green().bold());
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
            RenameResult {
                source: PathBuf::from("/tmp/photo.jpg"),
                target: PathBuf::from("/tmp/renamed.jpg"),
                success: true,
                error: None,
            },
            RenameResult {
                source: PathBuf::from("/tmp/doc.txt"),
                target: PathBuf::from("/tmp/doc.txt"),
                success: true,
                error: None,
            },
            RenameResult {
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

    #[test]
    fn display_width_counts_columns() {
        assert_eq!(display_width("abc"), 3);
        assert_eq!(display_width("中文"), 4);
        assert_eq!(display_width("😀"), 2);
    }

    #[test]
    fn pad_uses_display_width() {
        assert_eq!(pad("ab", 5), "ab   ");
        // Two CJK characters occupy four columns.
        assert_eq!(pad("中文", 6), "中文  ");
        assert_eq!(pad("x", 1), "x");
        // Shorter widths never truncate.
        assert_eq!(pad("x", 0), "x");
    }

    #[test]
    fn max_width_uses_display_width_and_clamps() {
        let short = vec![Cow::Borrowed("中文")];
        assert_eq!(max_width(&short), 20);

        // 30 CJK characters = 60 display columns (would be 30 by char count).
        let wide = vec![Cow::Owned("中".repeat(30))];
        assert_eq!(max_width(&wide), 60);

        let long = vec![Cow::Owned("a".repeat(100))];
        assert_eq!(max_width(&long), 60);
    }

    #[test]
    fn summarize_excludes_unchanged_from_renamed() {
        let results = vec![
            RenameResult {
                source: PathBuf::from("/tmp/a.txt"),
                target: PathBuf::from("/tmp/b.txt"),
                success: true,
                error: None,
            },
            RenameResult {
                source: PathBuf::from("/tmp/c.txt"),
                target: PathBuf::from("/tmp/c.txt"),
                success: true,
                error: None,
            },
            RenameResult {
                source: PathBuf::from("/tmp/d.txt"),
                target: PathBuf::from("/tmp/e.txt"),
                success: false,
                error: Some("denied".into()),
            },
        ];

        let summary = summarize(&results);
        assert_eq!(summary.renamed, 1);
        assert_eq!(summary.skipped, 1);
        assert_eq!(summary.failed, 1);
    }

    #[test]
    fn summarize_all_unchanged() {
        let results = vec![
            RenameResult {
                source: PathBuf::from("/tmp/a.txt"),
                target: PathBuf::from("/tmp/a.txt"),
                success: true,
                error: None,
            },
            RenameResult {
                source: PathBuf::from("/tmp/b.txt"),
                target: PathBuf::from("/tmp/b.txt"),
                success: true,
                error: None,
            },
        ];

        let summary = summarize(&results);
        assert_eq!(summary.renamed, 0);
        assert_eq!(summary.skipped, 2);
        assert_eq!(summary.failed, 0);
    }
}
