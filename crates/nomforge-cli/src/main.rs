mod args;

use args::Cli;
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        args::Commands::Rename(args) => {
            println!("Rename command: dir={}", args.dir);
            println!("  (not yet implemented)");
        }
        args::Commands::Undo(_args) => {
            println!("Undo command");
            println!("  (not yet implemented)");
        }
    }
}
