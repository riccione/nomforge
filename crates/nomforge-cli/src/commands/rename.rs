use anyhow::Result;
use nomforge_core::{Case, RenameRule, ScanOptions, SeqPosition};

use crate::args::{CaseArg, CounterPositionArg, RenameArgs};

/// Convert CLI args into a list of rename rules.
pub fn build_rules(args: &RenameArgs) -> Result<Vec<RenameRule>> {
    let mut rules = Vec::new();

    // Replacement flags are meaningless without their matcher; rejecting them
    // avoids silently dropping what the user asked for.
    if args.replace.is_some() && args.find.is_none() {
        anyhow::bail!("--replace requires --find");
    }
    if args.replacement.is_some() && args.regex.is_none() {
        anyhow::bail!("--replacement requires --regex");
    }

    // Find & Replace (plain text); without --replace the matches are removed
    if let Some(find) = &args.find {
        rules.push(RenameRule::FindReplace {
            find: find.clone(),
            replace: args.replace.clone().unwrap_or_default(),
        });
    }

    // Regex Replace; without --replacement the matches are removed
    if let Some(pattern) = &args.regex {
        rules.push(RenameRule::RegexReplace {
            pattern: pattern.clone(),
            replacement: args.replacement.clone().unwrap_or_default(),
        });
    }

    // Prefix
    if let Some(prefix) = &args.prefix {
        rules.push(RenameRule::Prefix(prefix.clone()));
    }

    // Suffix
    if let Some(suffix) = &args.suffix {
        rules.push(RenameRule::Suffix(suffix.clone()));
    }

    // Remove Text
    if let Some(text) = &args.remove {
        rules.push(RenameRule::RemoveText(text.clone()));
    }

    // Case Transform
    if let Some(case) = args.case {
        let case = match case {
            CaseArg::Upper => Case::Upper,
            CaseArg::Lower => Case::Lower,
            CaseArg::Title => Case::Title,
        };
        rules.push(RenameRule::CaseTransform(case));
    }

    // Number Sequence (only if counter_start or counter_padding is explicitly set)
    if args.counter_start.is_some() || args.counter_padding.is_some() {
        let position = match args.counter_position {
            CounterPositionArg::Prefix => SeqPosition::Prefix,
            CounterPositionArg::Suffix => SeqPosition::Suffix,
            CounterPositionArg::Replace => SeqPosition::ReplaceStem,
        };
        rules.push(RenameRule::NumberSequence {
            start: args.counter_start.unwrap_or(1),
            padding: args.counter_padding.unwrap_or(0),
            position,
        });
    }

    // Change the file extension
    if let Some(new_ext) = &args.new_ext {
        rules.push(RenameRule::ChangeExtension {
            new_ext: Some(new_ext.clone()),
        });
    }

    if rules.is_empty() {
        anyhow::bail!(
            "No rename rules specified. Use --find, --regex, --prefix, --suffix, --remove, --case, or --new-ext."
        );
    }

    Ok(rules)
}

/// Build scan options from CLI args.
pub fn build_scan_options(args: &RenameArgs) -> ScanOptions {
    ScanOptions {
        recursive: args.recursive,
        include_pattern: args.include.clone(),
        exclude_pattern: args.exclude.clone(),
        extensions: args.ext.clone(),
        include_hidden: args.hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_args() -> RenameArgs {
        RenameArgs {
            dir: std::path::PathBuf::from("."),
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
        }
    }

    #[test]
    fn build_rules_find_replace() {
        let args = RenameArgs {
            find: Some("DSC".into()),
            replace: Some("photo".into()),
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert_eq!(rules.len(), 1);
        assert!(
            matches!(&rules[0], RenameRule::FindReplace { find, replace } if find == "DSC" && replace == "photo")
        );
    }

    #[test]
    fn build_rules_replace_without_find_errors() {
        let args = RenameArgs {
            replace: Some("photo".into()),
            ..default_args()
        };
        let err = build_rules(&args).unwrap_err();
        assert!(
            err.to_string().contains("--replace requires --find"),
            "err: {err}"
        );
    }

    #[test]
    fn build_rules_replacement_without_regex_errors() {
        let args = RenameArgs {
            replacement: Some("img_$1".into()),
            ..default_args()
        };
        let err = build_rules(&args).unwrap_err();
        assert!(
            err.to_string().contains("--replacement requires --regex"),
            "err: {err}"
        );
    }

    #[test]
    fn build_rules_find_without_replace_deletes() {
        let args = RenameArgs {
            find: Some("DSC".into()),
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert_eq!(rules.len(), 1);
        assert!(
            matches!(&rules[0], RenameRule::FindReplace { find, replace } if find == "DSC" && replace.is_empty())
        );
    }

    #[test]
    fn build_rules_regex_without_replacement_deletes() {
        let args = RenameArgs {
            regex: Some(r"\d+".into()),
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert_eq!(rules.len(), 1);
        assert!(
            matches!(&rules[0], RenameRule::RegexReplace { pattern, replacement } if pattern == r"\d+" && replacement.is_empty())
        );
    }

    #[test]
    fn build_rules_regex() {
        let args = RenameArgs {
            regex: Some(r"(\d+)".into()),
            replacement: Some("img_$1".into()),
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert_eq!(rules.len(), 1);
        assert!(
            matches!(&rules[0], RenameRule::RegexReplace { pattern, replacement } if pattern == r"(\d+)" && replacement == "img_$1")
        );
    }

    #[test]
    fn build_rules_prefix_suffix() {
        let args = RenameArgs {
            prefix: Some("pre_".into()),
            suffix: Some("_suf".into()),
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert_eq!(rules.len(), 2);
        assert!(matches!(&rules[0], RenameRule::Prefix(p) if p == "pre_"));
        assert!(matches!(&rules[1], RenameRule::Suffix(s) if s == "_suf"));
    }

    #[test]
    fn build_rules_remove() {
        let args = RenameArgs {
            remove: Some(" ".into()),
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(&rules[0], RenameRule::RemoveText(t) if t == " "));
    }

    #[test]
    fn build_rules_case() {
        let args = RenameArgs {
            case: Some(CaseArg::Upper),
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(&rules[0], RenameRule::CaseTransform(Case::Upper)));
    }

    #[test]
    fn build_rules_counter() {
        let args = RenameArgs {
            counter_start: Some(1),
            counter_padding: Some(3),
            counter_position: CounterPositionArg::Prefix,
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(
            &rules[0],
            RenameRule::NumberSequence {
                start: 1,
                padding: 3,
                position: SeqPosition::Prefix
            }
        ));
    }

    #[test]
    fn build_rules_counter_position_suffix() {
        let args = RenameArgs {
            counter_padding: Some(3),
            counter_position: CounterPositionArg::Suffix,
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert!(matches!(
            &rules[0],
            RenameRule::NumberSequence {
                position: SeqPosition::Suffix,
                ..
            }
        ));
    }

    #[test]
    fn build_rules_counter_position_replace() {
        let args = RenameArgs {
            counter_padding: Some(3),
            counter_position: CounterPositionArg::Replace,
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert!(matches!(
            &rules[0],
            RenameRule::NumberSequence {
                position: SeqPosition::ReplaceStem,
                ..
            }
        ));
    }

    #[test]
    fn build_rules_multiple_combined() {
        let args = RenameArgs {
            prefix: Some("vacation_".into()),
            case: Some(CaseArg::Lower),
            counter_start: Some(1),
            counter_padding: Some(3),
            counter_position: CounterPositionArg::Prefix,
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert_eq!(rules.len(), 3);
        assert!(matches!(&rules[0], RenameRule::Prefix(_)));
        assert!(matches!(&rules[1], RenameRule::CaseTransform(Case::Lower)));
        assert!(matches!(&rules[2], RenameRule::NumberSequence { .. }));
    }

    #[test]
    fn build_rules_no_rules_errors() {
        let args = default_args();
        assert!(build_rules(&args).is_err());
    }

    #[test]
    fn build_rules_change_extension() {
        let args = RenameArgs {
            new_ext: Some("jpg".into()),
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert_eq!(rules.len(), 1);
        assert!(
            matches!(&rules[0], RenameRule::ChangeExtension { new_ext: Some(ext) } if ext == "jpg")
        );
    }

    #[test]
    fn build_rules_change_extension_empty_removes_it() {
        let args = RenameArgs {
            new_ext: Some(String::new()),
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert_eq!(rules.len(), 1);
        assert!(
            matches!(&rules[0], RenameRule::ChangeExtension { new_ext: Some(ext) } if ext.is_empty())
        );
    }

    #[test]
    fn build_rules_counter_explicit_defaults() {
        // User explicitly passes --counter-start 1 --counter-padding 0
        // These are the defaults, but counter should still be added
        let args = RenameArgs {
            counter_start: Some(1),
            counter_padding: Some(0),
            counter_position: CounterPositionArg::Prefix,
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(
            &rules[0],
            RenameRule::NumberSequence {
                start: 1,
                padding: 0,
                position: SeqPosition::Prefix
            }
        ));
    }

    #[test]
    fn build_scan_options_defaults() {
        let args = default_args();
        let opts = build_scan_options(&args);
        assert!(!opts.recursive);
        assert!(!opts.include_hidden);
        assert!(opts.extensions.is_none());
        assert!(opts.include_pattern.is_none());
        assert!(opts.exclude_pattern.is_none());
    }

    #[test]
    fn build_scan_options_with_values() {
        let args = RenameArgs {
            recursive: true,
            hidden: true,
            ext: Some(vec!["txt".into(), "jpg".into()]),
            include: Some("^file".into()),
            exclude: Some(r"\.bak$".into()),
            ..default_args()
        };
        let opts = build_scan_options(&args);
        assert!(opts.recursive);
        assert!(opts.include_hidden);
        assert_eq!(opts.extensions.as_ref().unwrap().len(), 2);
        assert_eq!(opts.include_pattern.as_ref().unwrap(), "^file");
        assert_eq!(opts.exclude_pattern.as_ref().unwrap(), r"\.bak$");
    }
}
