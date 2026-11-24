use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

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

/// Case transformation choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CaseArg {
    Upper,
    Lower,
    Title,
}

/// Counter placement choice.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum CounterPositionArg {
    #[default]
    Prefix,
    Suffix,
    Replace,
}

/// Arguments for the rename command.
#[derive(Parser, Debug)]
pub struct RenameArgs {
    /// Target directory to scan
    #[arg(short, long)]
    pub dir: PathBuf,

    /// Plain text to find in filename
    #[arg(long)]
    pub find: Option<String>,

    /// Replacement text (requires --find)
    #[arg(long)]
    pub replace: Option<String>,

    /// Regex pattern to match in filename
    #[arg(short, long)]
    pub regex: Option<String>,

    /// Replacement string for regex (requires --regex, supports $1, $2, etc.)
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
    #[arg(long, value_enum, ignore_case = true)]
    pub case: Option<CaseArg>,

    /// Change the file extension (without dot; empty string removes it)
    #[arg(long)]
    pub new_ext: Option<String>,

    /// Counter start value (enables counter rule when set)
    #[arg(long)]
    pub counter_start: Option<usize>,

    /// Counter zero-padding width (enables counter rule when set)
    #[arg(long)]
    pub counter_padding: Option<usize>,

    /// Counter position: prefix, suffix, replace
    #[arg(long, value_enum, ignore_case = true, default_value = "prefix")]
    pub counter_position: CounterPositionArg,

    /// Filter by file extension (repeatable or comma-separated)
    #[arg(short, long, value_delimiter = ',')]
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

    /// Include hidden files and directories (names starting with .)
    #[arg(long)]
    pub hidden: bool,

    /// Actually apply renames (default is dry-run)
    #[arg(short, long)]
    pub apply: bool,

    /// Apply even when conflicts are detected; conflicting renames fail
    /// instead of overwriting
    #[arg(long)]
    pub force: bool,

    /// Skip logging to undo history
    #[arg(long)]
    pub no_undo: bool,

    /// Custom undo log file path
    #[arg(long)]
    pub history_file: Option<PathBuf>,

    /// Show detailed output
    #[arg(short, long)]
    pub verbose: bool,
}

/// Arguments for the undo command.
#[derive(Parser, Debug)]
pub struct UndoArgs {
    /// Custom undo log file path
    #[arg(long)]
    pub history_file: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};

    #[test]
    fn cli_structures_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn ext_accepts_comma_separated_values() {
        let cli =
            Cli::try_parse_from(["nomforge", "rename", "--dir", ".", "--ext", "txt,jpg"]).unwrap();

        match cli.command {
            Commands::Rename(args) => {
                assert_eq!(args.ext, Some(vec!["txt".to_string(), "jpg".to_string()]));
            }
            _ => panic!("expected rename command"),
        }
    }

    #[test]
    fn case_accepts_uppercase_input() {
        let cli =
            Cli::try_parse_from(["nomforge", "rename", "--dir", ".", "--case", "UPPER"]).unwrap();

        match cli.command {
            Commands::Rename(args) => assert_eq!(args.case, Some(CaseArg::Upper)),
            _ => panic!("expected rename command"),
        }
    }

    #[test]
    fn invalid_enum_values_are_rejected_at_parse() {
        assert!(
            Cli::try_parse_from(["nomforge", "rename", "--dir", ".", "--case", "sideways"])
                .is_err()
        );
        // Also rejected when no counter flags are given.
        assert!(
            Cli::try_parse_from([
                "nomforge",
                "rename",
                "--dir",
                ".",
                "--counter-position",
                "sideways",
            ])
            .is_err()
        );
    }

    #[test]
    fn rename_args_defaults() {
        let args = RenameArgs {
            dir: PathBuf::from("."),
            find: None,
            replace: None,
            regex: None,
            replacement: None,
            prefix: None,
            suffix: None,
            remove: None,
            case: None,
            new_ext: None,
            counter_start: None,
            counter_padding: None,
            counter_position: CounterPositionArg::Prefix,
            ext: None,
            include: None,
            exclude: None,
            recursive: false,
            hidden: false,
            apply: false,
            force: false,
            no_undo: false,
            history_file: None,
            verbose: false,
        };
        assert!(args.counter_start.is_none());
        assert!(args.counter_padding.is_none());
        assert!(!args.apply);
        assert!(!args.force);
    }
}
