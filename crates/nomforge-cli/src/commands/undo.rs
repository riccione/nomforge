use crate::args;
use colored::Colorize;

pub fn run(args: &args::UndoArgs) -> anyhow::Result<()> {
    let history_path = args
        .history_file
        .as_deref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(nomforge_core::default_undo_log_path);

    let count = nomforge_core::undo_count(&history_path)?;
    if count == 0 {
        println!("  No undo history found.");
        return Ok(());
    }

    println!("  Undoing last batch ({} total batches)...", count);
    let reverted = nomforge_core::revert_last(&history_path)?;
    println!(
        "  {}",
        format!("Reverted {} file(s).", reverted).green().bold()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_run_no_history() {
        let args = args::UndoArgs {
            history_file: Some("/tmp/nonexistent_nomforge_undo_test.json".into()),
        };
        // Should succeed with no history
        let result = run(&args);
        assert!(result.is_ok());
    }
}
