mod args;
mod commands;

use args::Cli;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        args::Commands::Rename(args) => {
            let rules = commands::rename::build_rules(args)?;
            let scan_options = commands::rename::build_scan_options(args);

            println!("Dir: {}", args.dir);
            println!("Rules: {:?}", rules);
            println!("Scan options: {:?}", scan_options);
            println!("  (apply not yet wired up)");
        }
        args::Commands::Undo(_args) => {
            println!("Undo command");
            println!("  (not yet implemented)");
        }
    }

    Ok(())
}
