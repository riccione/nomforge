## [0.4.0] - 2026-10-06

### Bug Fixes

- *(engine)* Skip disambiguation for no-op rename plans
- *(core)* Exclude no-op renames from undo log
- *(cli)* Exclude unchanged files from failure rename count
- *(gui)* Count only actual renames in apply status
- *(core)* Refuse overwriting existing target on apply
- *(gui)* Correct conflict modal overwrite copy
- *(core)* Reject invalid rule output stems during planning
- *(gui)* Increase default font size by 2pt
- *(core)* Refuse clobbering recreated source on undo
- *(core)* Retain undo history when revert fails
- *(core)* Guard counter sequence against overflow
- *(core)* Bound counter padding width

### Documentation

- *(readme)* Note no-op plans are never disambiguated
- *(readme)* Document conflict abort and --force
- *(readme)* Add release instructions

### Features

- *(cli)* Block conflicting apply without --force
- *(release)* Configure cargo-release and git-cliff changelog automation

### Miscellaneous Tasks

- Add rust-toolchain.toml
- *(gui)* Restore nomforge-gui binary name
- *(cli)* Drop redundant bin block
- Adopt reusable workflows for release and pin CI workflow SHA (#77)
- *(release)* Generate changelog separately

### Testing

- *(core)* Use real files in no-op tests that dodged disambiguation
- *(core)* Rewrite engine conflict tests around disambiguation
## [0.3.0] - 2026-07-09

### Bug Fixes

- *(scanner)* Propagate walkdir errors instead of silently swallowing them
- *(engine)* Eliminate double-application of extension rules
- *(undo)* Eliminate TOCTOU race condition in revert_last
- *(gui)* Separate extension filter from extension change rule
- *(gui)* Reuse cached files in preview/apply instead of re-scanning
- *(gui)* Preserve plans after apply so preview table shows results
- *(undo)* Make default path portable across platforms
- *(cli)* Distinguish explicit counter args from defaults
- *(core)* Validate extension string rejects path traversal
- *(gui)* Remove global #[allow(dead_code)] and wire up counter
- *(gui)* Remove module-level allow(dead_code) from conflict_badge
- *(cli)* Use dynamic column widths for preview and results tables
- *(core)* Prevent UTF-8 panic in truncate_stem
- *(core)* Use idiomatic byte index comparison in truncate_stem
- *(core)* Change apply_extension signature to Option<&str>

### Documentation

- *(conflict)* Document TOCTOU race in TargetExists detection

### Features

- *(gui)* Show version in window title
- *(gui)* Add version number in header
- *(gui)* Add undo button with confirmation modal
- *(gui)* Add simple/advanced mode toggle for regex fields
- *(gui)* Wire conflict detection into apply flow
- *(core)* Add display module for truncation and disambiguation
- *(gui)* Add conflict detection to preview pipeline

### Miscellaneous Tasks

- Update workspace resolver to version 3
- Add rustfmt.toml and clippy.toml configuration

### Performance

- *(engine)* Pre-compile regex patterns once in RenameEngine
- *(core)* Add regex caching to RenameContext

### Refactor

- *(core)* Extract RegexCache and simplify RenameContext

### Testing

- *(engine)* Add tests for stem-modifying rules combined with extension change
- *(gui)* Add tests for apply undo log integration
## [0.2.0] - 2026-07-01

### Features

- *(gui)* Implement undo logging in GUI #36

### Miscellaneous Tasks

- Bump version to v0.2.0
## [0.1.0] - 2026-07-01

### Bug Fixes

- *(gui)* Prevent fullscreen window on launch
- *(test)* Correct binary path in CLI integration test helper

### Documentation

- Update README with usage, contributing, and license info
- Fix README inaccuracies

### Features

- *(core)* Add error types and rule trait with all rule variants
- *(core)* Extract FindReplace rule into separate module
- *(core)* Extract Prefix + Suffix rules into prefix_suffix module
- *(core)* Extract CaseTransform rule into case_transform module
- *(core)* Extract Counter rule into counter module
- *(core)* Extract RemoveText rule into remove_text module
- *(core)* Extract Extension rule into extension module
- *(core)* Extract RegexReplace rule into regex_replace module
- *(core)* Implement RenameEngine with plan + apply + conflict detection
- *(core)* Implement scanner with recursive walk and filtering
- *(core)* Extract conflict detection into separate conflict module
- *(core)* Implement undo support with JSON log and revert
- *(cli)* Define clap args with Cli, RenameArgs, UndoArgs
- *(cli)* Implement args to Vec<RenameRule> mapping
- *(cli)* Implement preview table and colored output
- *(cli)* Extract undo logic into commands/undo.rs
- *(gui)* Scaffold eframe app with State struct
- *(gui)* Extract folder picker into panels/folder_picker.rs
- *(gui)* Extract rule builder into panels/rule_builder.rs
- *(core)* Add proactive filename length validation
- *(gui)* Add preview table panel
- *(gui)* Add status bar panel
- *(gui)* Add reusable widgets (rule_card, conflict_badge)

### Miscellaneous Tasks

- Scaffold workspace with three crates
- Add a basic CI workflow for PR and pushes
- Format source code with cargo fmt
- Upgrade checkout action to v7
- Use shared reusable Rust workflow
- Fix typo in the name of the repo
- Move package metadata to workspace and optimize release profile
- Bump reusable ci to v3
- Add release workflow for CLI and GUI binaries
- Extract binary names to top-level env vars

### Testing

- *(core)* Add rename workflow integration tests
- *(core)* Add undo workflow integration tests
- *(core)* Add remaining core integration tests
- *(cli)* Add CLI integration tests
