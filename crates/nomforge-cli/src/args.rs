use clap::{Parser, Subcommand};

/// nomforge — bulk file renamer
#[derive(Parser, Debug)]
#[command(name = "nomforge", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Rename files using rules
    Rename(Box<RenameArgs>),
    /// Undo the last rename operation
    Undo(UndoArgs),
}

/// Arguments for the rename command.
#[derive(Parser, Debug)]
pub struct RenameArgs {
    /// Target directory to scan
    #[arg(short, long)]
    pub dir: String,

    /// Plain text to find in filename
    #[arg(long)]
    pub find: Option<String>,

    /// Replacement text (pairs with --find)
    #[arg(long)]
    pub replace: Option<String>,

    /// Regex pattern to match in filename
    #[arg(short, long)]
    pub regex: Option<String>,

    /// Replacement string for regex (pairs with --regex, supports $1, $2, etc.)
    #[arg(long)]
    pub replacement: Option<String>,

    /// Add prefix to filename
    #[arg(long)]
    pub prefix: Option<String>,

    /// Add suffix to filename
    #[arg(long)]
    pub suffix: Option<String>,

    /// Remove all occurrences of this text
    #[arg(long)]
    pub remove: Option<String>,

    /// Transform case: upper, lower, title
    #[arg(long)]
    pub case: Option<String>,

    /// Counter start value
    #[arg(long, default_value_t = 1)]
    pub counter_start: usize,

    /// Counter zero-padding width
    #[arg(long, default_value_t = 0)]
    pub counter_padding: usize,

    /// Counter position: prefix, suffix, replace
    #[arg(long, default_value = "prefix")]
    pub counter_position: String,

    /// Filter by file extension (repeatable)
    #[arg(short, long)]
    pub ext: Option<Vec<String>>,

    /// Include only files matching this regex pattern
    #[arg(short, long)]
    pub include: Option<String>,

    /// Exclude files matching this regex pattern
    #[arg(long)]
    pub exclude: Option<String>,

    /// Scan subdirectories recursively
    #[arg(short = 'R', long)]
    pub recursive: bool,

    /// Include hidden files (starting with .)
    #[arg(long)]
    pub hidden: bool,

    /// Actually apply renames (default is dry-run)
    #[arg(short, long)]
    pub apply: bool,

    /// Skip logging to undo history
    #[arg(long)]
    pub no_undo: bool,

    /// Custom undo log file path
    #[arg(long)]
    pub history_file: Option<String>,

    /// Show detailed output
    #[arg(short, long)]
    pub verbose: bool,
}

/// Arguments for the undo command.
#[derive(Parser, Debug)]
pub struct UndoArgs {
    /// Custom undo log file path
    #[arg(long)]
    pub history_file: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_structures_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn rename_args_defaults() {
        let args = RenameArgs {
            dir: ".".into(),
            find: None,
            replace: None,
            regex: None,
            replacement: None,
            prefix: None,
            suffix: None,
            remove: None,
            case: None,
            counter_start: 1,
            counter_padding: 0,
            counter_position: "prefix".into(),
            ext: None,
            include: None,
            exclude: None,
            recursive: false,
            hidden: false,
            apply: false,
            no_undo: false,
            history_file: None,
            verbose: false,
        };
        assert_eq!(args.counter_start, 1);
        assert_eq!(args.counter_padding, 0);
        assert!(!args.apply);
    }
}
