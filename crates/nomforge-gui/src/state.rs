use std::path::PathBuf;

use nomforge_core::{RenamePlan, RenameResult, RenameRule};

/// Application state for the nomforge GUI.
#[allow(dead_code)]
pub struct State {
    /// Target directory to scan.
    pub dir: String,

    /// Rule configuration.
    pub find: String,
    pub replace: String,
    pub regex: String,
    pub replacement: String,
    pub prefix: String,
    pub suffix: String,
    pub remove: String,
    pub case: String,
    pub counter_start: usize,
    pub counter_padding: usize,
    pub counter_position: String,

    /// File filtering.
    pub filter_ext: String,
    pub include: String,
    pub exclude: String,
    pub recursive: bool,
    pub hidden: bool,

    /// Extension change rule.
    pub ext_change: String,

    /// Undo settings.
    pub history_file: String,
    pub no_undo: bool,
    pub show_undo_modal: bool,

    /// Output / results.
    pub verbose: bool,

    /// Files found by scanner.
    pub files: Vec<PathBuf>,

    /// Generated rename plans.
    pub plans: Vec<RenamePlan>,

    /// Results after applying renames.
    pub results: Vec<RenameResult>,

    /// Status message.
    pub status: String,

    /// Whether a rename has been applied.
    pub applied: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            dir: String::new(),
            find: String::new(),
            replace: String::new(),
            regex: String::new(),
            replacement: String::new(),
            prefix: String::new(),
            suffix: String::new(),
            remove: String::new(),
            case: String::new(),
            counter_start: 1,
            counter_padding: 0,
            counter_position: "prefix".into(),
            filter_ext: String::new(),
            include: String::new(),
            exclude: String::new(),
            recursive: false,
            hidden: false,
            ext_change: String::new(),
            history_file: String::new(),
            no_undo: false,
            show_undo_modal: false,
            verbose: false,
            files: Vec::new(),
            plans: Vec::new(),
            results: Vec::new(),
            status: String::from("Ready"),
            applied: false,
        }
    }
}

impl State {
    /// Build rules from current state.
    pub fn build_rules(&self) -> nomforge_core::Result<Vec<RenameRule>> {
        let mut rules = Vec::new();

        if !self.find.is_empty() || !self.replace.is_empty() {
            rules.push(RenameRule::FindReplace {
                find: self.find.clone(),
                replace: self.replace.clone(),
            });
        }

        if !self.regex.is_empty() {
            rules.push(RenameRule::RegexReplace {
                pattern: self.regex.clone(),
                replacement: self.replacement.clone(),
            });
        }

        if !self.prefix.is_empty() {
            rules.push(RenameRule::Prefix(self.prefix.clone()));
        }

        if !self.suffix.is_empty() {
            rules.push(RenameRule::Suffix(self.suffix.clone()));
        }

        if !self.remove.is_empty() {
            rules.push(RenameRule::RemoveText(self.remove.clone()));
        }

        if !self.case.is_empty() {
            let case = match self.case.to_lowercase().as_str() {
                "upper" => nomforge_core::Case::Upper,
                "lower" => nomforge_core::Case::Lower,
                "title" => nomforge_core::Case::Title,
                _ => {
                    return Err(nomforge_core::NomforgeError::Conflict(format!(
                        "Invalid case: {}",
                        self.case
                    )));
                }
            };
            rules.push(RenameRule::CaseTransform(case));
        }

        if !self.ext_change.is_empty() {
            rules.push(RenameRule::ChangeExtension {
                new_ext: Some(self.ext_change.clone()),
            });
        }

        Ok(rules)
    }

    /// Build scan options from current state.
    pub fn build_scan_options(&self) -> nomforge_core::ScanOptions {
        let extensions = if self.filter_ext.is_empty() {
            None
        } else {
            let exts: Vec<String> = self
                .filter_ext
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if exts.is_empty() { None } else { Some(exts) }
        };

        nomforge_core::ScanOptions {
            extensions,
            recursive: self.recursive,
            include_pattern: if self.include.is_empty() {
                None
            } else {
                Some(self.include.clone())
            },
            exclude_pattern: if self.exclude.is_empty() {
                None
            } else {
                Some(self.exclude.clone())
            },
            include_hidden: self.hidden,
        }
    }

    /// Reset results and plans.
    pub fn reset_output(&mut self) {
        self.files.clear();
        self.plans.clear();
        self.results.clear();
        self.applied = false;
        self.status = "Ready".into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state() {
        let state = State::default();
        assert!(state.dir.is_empty());
        assert_eq!(state.counter_start, 1);
        assert!(!state.recursive);
        assert!(!state.hidden);
        assert!(state.files.is_empty());
        assert!(state.plans.is_empty());
        assert!(state.results.is_empty());
        assert_eq!(state.status, "Ready");
    }

    #[test]
    fn build_rules_empty() {
        let state = State::default();
        let rules = state.build_rules().unwrap();
        assert!(rules.is_empty());
    }

    #[test]
    fn build_rules_find_replace() {
        let state = State {
            find: "old".into(),
            replace: "new".into(),
            ..Default::default()
        };
        let rules = state.build_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(rules[0], RenameRule::FindReplace { .. }));
    }

    #[test]
    fn build_rules_regex() {
        let state = State {
            regex: r"(\d+)".into(),
            replacement: "$1".into(),
            ..Default::default()
        };
        let rules = state.build_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(rules[0], RenameRule::RegexReplace { .. }));
    }

    #[test]
    fn build_rules_prefix() {
        let state = State {
            prefix: "pre_".into(),
            ..Default::default()
        };
        let rules = state.build_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(rules[0], RenameRule::Prefix(_)));
    }

    #[test]
    fn build_rules_suffix() {
        let state = State {
            suffix: "_suf".into(),
            ..Default::default()
        };
        let rules = state.build_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(rules[0], RenameRule::Suffix(_)));
    }

    #[test]
    fn build_rules_remove() {
        let state = State {
            remove: "copy".into(),
            ..Default::default()
        };
        let rules = state.build_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(rules[0], RenameRule::RemoveText(_)));
    }

    #[test]
    fn build_rules_case_upper() {
        let state = State {
            case: "upper".into(),
            ..Default::default()
        };
        let rules = state.build_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(
            rules[0],
            RenameRule::CaseTransform(nomforge_core::Case::Upper)
        ));
    }

    #[test]
    fn build_rules_case_lower() {
        let state = State {
            case: "lower".into(),
            ..Default::default()
        };
        let rules = state.build_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(
            rules[0],
            RenameRule::CaseTransform(nomforge_core::Case::Lower)
        ));
    }

    #[test]
    fn build_rules_case_title() {
        let state = State {
            case: "title".into(),
            ..Default::default()
        };
        let rules = state.build_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(
            rules[0],
            RenameRule::CaseTransform(nomforge_core::Case::Title)
        ));
    }

    #[test]
    fn build_rules_invalid_case() {
        let state = State {
            case: "invalid".into(),
            ..Default::default()
        };
        let result = state.build_rules();
        assert!(result.is_err());
    }

    #[test]
    fn build_rules_extension() {
        let state = State {
            ext_change: "txt".into(),
            ..Default::default()
        };
        let rules = state.build_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(rules[0], RenameRule::ChangeExtension { .. }));
    }

    #[test]
    fn build_rules_multiple() {
        let state = State {
            prefix: "pre_".into(),
            case: "upper".into(),
            ext_change: "md".into(),
            ..Default::default()
        };
        let rules = state.build_rules().unwrap();
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn build_scan_options_defaults() {
        let state = State::default();
        let opts = state.build_scan_options();
        assert!(opts.extensions.is_none());
        assert!(!opts.recursive);
        assert!(opts.include_pattern.is_none());
        assert!(opts.exclude_pattern.is_none());
        assert!(!opts.include_hidden);
    }

    #[test]
    fn build_scan_options_with_values() {
        let state = State {
            filter_ext: "txt,md".into(),
            recursive: true,
            include: "test_.*".into(),
            exclude: "backup".into(),
            hidden: true,
            ..Default::default()
        };
        let opts = state.build_scan_options();
        assert_eq!(opts.extensions, Some(vec!["txt".into(), "md".into()]));
        assert!(opts.recursive);
        assert_eq!(opts.include_pattern.as_deref(), Some("test_.*"));
        assert_eq!(opts.exclude_pattern.as_deref(), Some("backup"));
        assert!(opts.include_hidden);
    }

    #[test]
    fn reset_output() {
        let mut state = State {
            files: vec![PathBuf::from("/tmp/test.txt")],
            plans: vec![RenamePlan {
                source: PathBuf::from("/tmp/test.txt"),
                target: PathBuf::from("/tmp/test.md"),
            }],
            results: vec![],
            applied: true,
            status: "Done".into(),
            ..Default::default()
        };
        state.reset_output();
        assert!(state.files.is_empty());
        assert!(state.plans.is_empty());
        assert!(state.results.is_empty());
        assert!(!state.applied);
        assert_eq!(state.status, "Ready");
    }
}
