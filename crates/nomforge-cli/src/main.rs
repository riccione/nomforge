mod args;
mod commands;
mod output;

use args::Cli;
use clap::Parser;
use colored::Colorize;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        args::Commands::Rename(args) => {
            let rules = commands::rename::build_rules(args)?;
            let scan_options = commands::rename::build_scan_options(args);
            let dir = std::path::Path::new(&args.dir);

            // Scan files
            let files = nomforge_core::scan_files(dir, &scan_options)?;
            output::print_scan_summary(dir, files.len());

            if files.is_empty() {
                println!("  No files matched the given filters.");
                return Ok(());
            }

            // Generate plans
            let engine = nomforge_core::RenameEngine::new(rules);
            let plans = engine.plan(&files)?;

            // Check conflicts
            let conflicts = nomforge_core::detect_conflicts(&plans);
            output::print_conflicts(&conflicts);

            // Preview or apply
            if args.apply {
                let results = engine.apply(&plans)?;
                output::print_results(&results);

                // Log to undo history (unless --no-undo)
                if !args.no_undo {
                    let history_path = args
                        .history_file
                        .as_deref()
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(nomforge_core::default_undo_log_path);
                    nomforge_core::log_renames(&history_path, &results)?;
                    if args.verbose {
                        println!("  Undo history saved to: {}", history_path.display());
                    }
                }
            } else {
                output::print_preview(&plans);
                println!(
                    "  {}",
                    "Run with --apply to execute renames.".yellow().dimmed()
                );
            }
        }
        args::Commands::Undo(args) => {
            commands::undo::run(args)?;
        }
    }

    Ok(())
}
