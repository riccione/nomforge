<div align="center">

# nomforge

A cross-platform bulk file renaming tool with CLI and GUI interfaces, built in Rust.

Rename files using composable rules, preview changes before applying them, and
undo previous operations when needed.

[![CI](https://github.com/riccione/nomforge/actions/workflows/ci.yml/badge.svg)](https://github.com/riccione/nomforge/actions)
[![Release](https://img.shields.io/github/v/release/riccione/nomforge)](https://github.com/riccione/nomforge/releases)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

</div>

## Features

- **Multiple rename rules:** find and replace, prefix, suffix, case
  transformation, counters, text removal, extension changes, and regex
  replacement.
- **Rule chaining:** combine multiple rules into a single rename operation.
- **Dry-run by default:** preview proposed changes without modifying files.
- **Undo support:** revert the last batch of renames using a persistent JSON
  history log.
- **Conflict detection:** detect filename collisions before applying changes.
  Use `--force` to attempt non-conflicting operations; conflicting files are
  never overwritten.
- **Flexible filtering:** filter by extension, include/exclude regex patterns,
  scan recursively, and optionally include hidden files and directories.
- **Smart filename handling:** automatically truncate filenames exceeding OS
  limits and disambiguate existing targets with `_1`, `_2`, etc.
- **GUI preferences:** light, dark, and system themes, adjustable font size, and automatic scan-and-preview on folder selection. Settings are applied live and persisted between runs.

## Installation

Prebuilt binaries are available for Linux, macOS, and Windows on the [GitHub Releases](https://github.com/riccione/nomforge/releases) page. Both CLI and GUI versions are provided.

### Download a prebuilt binary (Recommended)

Choose the executable matching your operating system and architecture.

| Platform | CLI | GUI |
|---|---|---|
| Linux (x86_64) | [Download](https://github.com/riccione/nomforge/releases/latest/download/nomforge-cli_linux-x86_64) | [Download](https://github.com/riccione/nomforge/releases/latest/download/nomforge-gui_linux-x86_64) |
| macOS (Apple Silicon) | [Download](https://github.com/riccione/nomforge/releases/latest/download/nomforge-cli_macos-arm64) | [Download](https://github.com/riccione/nomforge/releases/latest/download/nomforge-gui_macos-arm64) |
| macOS (Intel) | [Download](https://github.com/riccione/nomforge/releases/latest/download/nomforge-cli_macos-x86_64) | [Download](https://github.com/riccione/nomforge/releases/latest/download/nomforge-gui_macos-x86_64) |
| Windows (x86_64) | [Download](https://github.com/riccione/nomforge/releases/latest/download/nomforge-cli_win-x86_64.exe) | [Download](https://github.com/riccione/nomforge/releases/latest/download/nomforge-gui_win-x86_64.exe) |

After downloading:

**Linux / macOS**

Make the downloaded executable runnable:

```bash
chmod +x nomforge-cli_linux-x86_64
./nomforge-cli_linux-x86_64 --help
```

For the GUI, run the downloaded `nomforge-gui` executable instead.

**Windows**

Run the downloaded `.exe` file. The CLI can be launched from PowerShell or Command Prompt.

### Verify downloads

Each release includes a `SHA256SUMS` file containing checksums for the binaries.

On Linux:

```bash
sha256sum -c SHA256SUMS
```

On macOS:

```bash
shasum -a 256 -c SHA256SUMS
```

On Windows PowerShell:

```powershell
Get-FileHash .\nomforge-cli_win-x86_64.exe -Algorithm SHA256
```

Compare the resulting hash with the corresponding entry in `SHA256SUMS`.

### Build from source

Requires **Rust 1.85+** (Edition 2024).

Clone the repository:

```bash
git clone https://github.com/riccione/nomforge.git
cd nomforge
```

Build both applications:

```bash
cargo build --release --workspace
```

The executables are available under `target/release/`.

Alternatively, run an application directly:

```bash
# CLI
cargo run --release -p nomforge-cli -- --help

# GUI
cargo run --release -p nomforge-gui
```

### Install the CLI using Cargo

You can also install the CLI directly from the repository:

```bash
cargo install --git https://github.com/riccione/nomforge.git --tag v0.4.0 --package nomforge-cli
```

This installs the `nomforge-cli` executable into your Cargo binary directory.

## Usage

### Basic renaming

All rename operations are dry-runs by default. Add `--apply` to actually rename files.

```bash
# Preview changes
nomforge-cli rename --dir ./photos --prefix "vacation_"

# Apply changes
nomforge-cli rename --dir ./photos --prefix "vacation_" --apply

# Transform filenames to uppercase
nomforge-cli rename --dir ./docs --case upper --apply

# Find and replace text
nomforge-cli rename --dir ./downloads --find "old" --replace "new" --apply

# Replace text using regular expressions
nomforge-cli rename --dir ./logs \
  --regex '(\d{4})-(\d{2})' \
  --replacement '${2}-${1}' \
  --apply

# Add sequential counters
nomforge-cli rename --dir ./exports \
  --counter-start 1 \
  --counter-padding 3 \
  --counter-position suffix \
  --apply

# Remove text from filenames
nomforge-cli rename --dir ./downloads --remove "copy" --apply
```

### File filtering

```bash
# Only process .txt files
nomforge-cli rename --dir ./docs --ext txt --case upper --apply

# Include subdirectories
nomforge-cli rename --dir ./project --recursive \
  --find "old" --replace "new" --apply

# Include and exclude files using regex patterns
nomforge-cli rename --dir ./photos \
  --include 'IMG_\d+' \
  --exclude 'backup' \
  --prefix "photo_" \
  --apply

# Include hidden files and directories
nomforge-cli rename --dir ./config --hidden \
  --find "." --replace "_" --apply
```

**Scanning behavior:**

- Symbolic links are not followed. Symlinked files and directories are ignored. A scan root that is itself a symlink is still traversed.
- Entries that cannot be read (for example, directories without sufficient permissions) are skipped with a warning. The remaining scan continues.

### Undo

Every successful rename batch is recorded in a persistent JSON history log unless `--no-undo` is specified.

```bash
# Undo the last batch
nomforge-cli undo

# Use a custom history file
nomforge-cli undo --history-file /path/to/undo.json
```

The default history location is:

| Platform | Location |
|---|---|
| Linux / macOS | `~/.local/share/nomforge/undo_log.json` |
| Windows | `%USERPROFILE%\.local\share\nomforge\undo_log.json` |

The history directory is created automatically when needed.

Additional safeguards:

- **Atomic writes:** history updates use a temporary file followed by a rename, protecting existing history from interrupted writes.
- **Corrupt log recovery:** unreadable history files are preserved as `undo_log.json.corrupt` (or `undo_log.json.corrupt.N`) before a fresh log is created.
- **Portable undo:** recorded paths are absolute, so undo works regardless of the current working directory.

## CLI Reference

### `nomforge-cli rename`

| Option | Short | Description |
|---|---|---|
| `--dir` | `-d` | Directory to scan |
| `--find` | | Plain text to find in filenames |
| `--replace` | | Replacement text (requires `--find`) |
| `--regex` | `-r` | Regular expression pattern |
| `--replacement` | | Regex replacement (requires `--regex`; supports `${1}`, `${2}`, etc.) |
| `--prefix` | | Add a prefix |
| `--suffix` | | Add a suffix |
| `--remove` | | Remove all occurrences of text |
| `--case` | | Transform case: `upper`, `lower`, or `title` |
| `--counter-start` | | Counter starting value (default: `1`) |
| `--counter-padding` | | Zero-padding width (default: `0`, maximum: `32`) |
| `--counter-position` | | Counter position: `prefix`, `suffix`, or `replace` (default: `prefix`) |
| `--ext` | `-e` | Filter by file extension (repeatable) |
| `--include` | `-i` | Include only filenames matching a regex |
| `--exclude` | | Exclude filenames matching a regex |
| `--recursive` | `-R` | Scan subdirectories recursively |
| `--hidden` | | Include hidden files and directories |
| `--apply` | `-a` | Apply changes (default: dry-run) |
| `--force` | | Attempt renames despite detected conflicts |
| `--no-undo` | | Skip undo history logging |
| `--history-file` | | Custom undo history file |
| `--verbose` | `-v` | Show detailed output |

**Exit status:** `rename --apply` returns a non-zero exit status if one or more renames fail. Successful operations are still recorded in the undo history.

**Regex replacement:** use brace syntax for capture group references to avoid ambiguity. For example, `${2}_${1}` instead of `$2_$1`, since `$2_` may be interpreted as a named group reference.

### `nomforge-cli undo`

| Option | Description |
|---|---|
| `--history-file` | Custom undo history file |

For the complete list of options, run:

```bash
nomforge-cli --help
nomforge-cli rename --help
nomforge-cli undo --help
```

## Architecture

nomforge is organized as a Rust workspace:

```text
nomforge/
├── crates/
│   ├── nomforge-core/    # Rename rules, engine, scanning, conflicts, undo
│   ├── nomforge-cli/     # Command-line interface (clap)
│   └── nomforge-gui/     # Graphical interface (eframe)
```

The core crate contains the renaming logic and is shared by the CLI and GUI interfaces.

## Contributing

Contributions, bug reports, and feature suggestions are welcome.

Before submitting a pull request, make sure the project passes the following checks:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```

For pull requests:

- Describe what changed and why.
- Link related issues or discussions.
- Include screenshots or recordings for GUI changes.
- Update documentation when necessary.

Use [Conventional Commits](https://www.conventionalcommits.org/) for commit messages. Examples:

```text
feat: add batch undo support
fix(engine): resolve conflict detection edge case
docs: update README
```

## License

Licensed under the [Apache License 2.0](LICENSE).
