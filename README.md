# nomforge

A bulk file renaming tool with CLI and GUI interfaces, built in Rust.

## Features

- **8 rename rules**: find/replace, prefix/suffix, case transform, counter, remove text, change extension, regex replace
- **Rule chaining**: combine multiple rules in a single pass
- **Dry-run by default**: preview all changes before applying
- **Undo support**: revert the last batch of renames via JSON log
- **Conflict detection**: warns when multiple files would collide
- **File filtering**: by extension, include/exclude regex patterns, recursive scan, hidden files

## Architecture

```
nomforge-core    — rename rules, engine, scanner, conflict detection, undo log
nomforge-cli     — command-line interface (clap)
nomforge-gui     — graphical interface (eframe)
```

## Installation

Requires **Rust edition 2024** (Rust 1.85+).

```bash
cargo install --path crates/nomforge-cli
```

This installs the `nomforge-cli` binary.

## Usage

### Rename files

```bash
# Dry-run (default) — preview changes without modifying files
nomforge-cli rename --dir ./photos --prefix "vacation_" --suffix "_2024"

# Apply renames
nomforge-cli rename --dir ./photos --prefix "vacation_" --apply

# Case transform
nomforge-cli rename --dir ./docs --case upper --apply

# Regex replace
nomforge-cli rename --dir ./logs --regex "(\d{4})-(\d{2})" --replacement "${2}-${1}" --apply

# Counter
nomforge-cli rename --dir ./exports --counter-start 1 --counter-padding 3 --counter-position suffix --apply

# Remove text
nomforge-cli rename --dir ./downloads --remove "copy" --apply
```

### File filtering

```bash
# Only .txt files
nomforge-cli rename --dir ./docs --ext txt --case upper --apply

# Recursive scan
nomforge-cli rename --dir ./project --recursive --find "old" --replace "new" --apply

# Include/exclude patterns
nomforge-cli rename --dir ./photos --include "IMG_\d+" --exclude "backup" --prefix "photo_" --apply

# Include hidden files
nomforge-cli rename --dir ./config --hidden --find "." --replace "_" --apply
```

### Undo

The undo log defaults to `~/.local/share/nomforge/undo_log.json`. The directory is created automatically on first use.

```bash
# Undo last batch
nomforge-cli undo

# Undo with custom history file
nomforge-cli undo --history-file /path/to/undo.json
```

## CLI Reference

### `nomforge-cli rename`

| Flag | Short | Description |
|------|-------|-------------|
| `--dir` | `-d` | Target directory to scan |
| `--find` | | Plain text to find in filename |
| `--replace` | | Replacement text (pairs with --find) |
| `--regex` | `-r` | Regex pattern to match |
| `--replacement` | | Replacement string for regex (supports `${1}`, `${2}`, etc.) |
| `--prefix` | | Add prefix to filename |
| `--suffix` | | Add suffix to filename |
| `--remove` | | Remove all occurrences of this text |
| `--case` | | Transform case: upper, lower, title |
| `--counter-start` | | Counter start value (default: 1) |
| `--counter-padding` | | Counter zero-padding width (default: 0) |
| `--counter-position` | | Counter position: prefix, suffix, replace (default: prefix) |
| `--ext` | `-e` | Filter by file extension (repeatable) |
| `--include` | `-i` | Include only files matching this regex |
| `--exclude` | | Exclude files matching this regex |
| `--recursive` | `-R` | Scan subdirectories recursively |
| `--hidden` | | Include hidden files |
| `--apply` | `-a` | Actually apply renames (default is dry-run) |
| `--no-undo` | | Skip logging to undo history |
| `--history-file` | | Custom undo log file path |
| `--verbose` | `-v` | Show detailed output |

> **Note on regex replacement:** Use brace syntax for capture group references to avoid ambiguity. For example, use `${2}_${1}` instead of `$2_$1`, since `$2_` would be parsed as a named group reference.

### `nomforge-cli undo`

| Flag | Description |
|------|-------------|
| `--history-file` | Custom undo log file path |

## How to Contribute

We welcome contributions to nomforge! Please ensure your work aligns with our formatting and workflow structures.

### Branch Naming & Commit Style

We enforce [Conventional Commits](https://www.conventionalcommits.org/) for both branch naming and commit messages.

- **Branches**: Use semantic prefixes followed by a short description (e.g., `feat/cli-output`, `fix/regex-escape`, `docs/update-readme`).
- **Commits**: Structure messages using a structural indicator (e.g., `feat: add counter rule`, `fix(engine): resolve conflict detection edge case`).

### Pull Request Format

When opening a Pull Request, include the following:

- **Clear Title**: Use a structured title (e.g., `feat: add batch undo support`).
- **Essence of Changes**: Briefly explain what was altered, why it was necessary, and how it was implemented.
- **Traceability Links**: Link to related issues, discussions, or documentation.
- **Visual Evidence**: Attach screenshots or recordings if the change affects UI.
- **Self-Check Checklist**:
  - [ ] `cargo check`
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all-targets -- -D warnings`
  - [ ] `cargo test --workspace`
  - [ ] Updated documentation if applicable

## License

This project is licensed under the terms of the Apache License 2.0. For the full legal text detailing permissions, limitations, and liabilities, please consult the complete [LICENSE](LICENSE) file included in this repository.
