use anyhow::Result;
use nomforge_core::{Case, RenameRule, ScanOptions, SeqPosition};

use crate::args::RenameArgs;

/// Convert CLI args into a list of rename rules.
pub fn build_rules(args: &RenameArgs) -> Result<Vec<RenameRule>> {
    let mut rules = Vec::new();

    // Find & Replace (plain text)
    if let (Some(find), replace) = (&args.find, &args.replace) {
        rules.push(RenameRule::FindReplace {
            find: find.clone(),
            replace: replace.clone().unwrap_or_default(),
        });
    }

    // Regex Replace
    if let (Some(pattern), replacement) = (&args.regex, &args.replacement) {
        rules.push(RenameRule::RegexReplace {
            pattern: pattern.clone(),
            replacement: replacement.clone().unwrap_or_default(),
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
    if let Some(case_str) = &args.case {
        let case = match case_str.to_lowercase().as_str() {
            "upper" => Case::Upper,
            "lower" => Case::Lower,
            "title" => Case::Title,
            _ => anyhow::bail!(
                "Invalid case transform '{}'. Expected: upper, lower, title",
                case_str
            ),
        };
        rules.push(RenameRule::CaseTransform(case));
    }

    // Number Sequence (only if counter_start or counter_padding is explicitly set)
    if args.counter_start.is_some() || args.counter_padding.is_some() {
        let position = match args.counter_position.to_lowercase().as_str() {
            "prefix" => SeqPosition::Prefix,
            "suffix" => SeqPosition::Suffix,
            "replace" => SeqPosition::ReplaceStem,
            _ => anyhow::bail!(
                "Invalid counter position '{}'. Expected: prefix, suffix, replace",
                args.counter_position
            ),
        };
        rules.push(RenameRule::NumberSequence {
            start: args.counter_start.unwrap_or(1),
            padding: args.counter_padding.unwrap_or(0),
            position,
        });
    }

    // Change Extension (if --ext is provided as a single value to change TO)
    // Note: --ext is currently used for filtering, not extension change
    // Extension change could be added as a separate flag later

    if rules.is_empty() {
        anyhow::bail!(
            "No rename rules specified. Use --find, --regex, --prefix, --suffix, --remove, or --case."
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
            dir: ".".into(),
            find: None,
            replace: None,
            regex: None,
            replacement: None,
            prefix: None,
            suffix: None,
            remove: None,
            case: None,
            counter_start: None,
            counter_padding: None,
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
            case: Some("upper".into()),
            ..default_args()
        };
        let rules = build_rules(&args).unwrap();
        assert_eq!(rules.len(), 1);
        assert!(matches!(&rules[0], RenameRule::CaseTransform(Case::Upper)));
    }

    #[test]
    fn build_rules_case_invalid() {
        let args = RenameArgs {
            case: Some("invalid".into()),
            ..default_args()
        };
        assert!(build_rules(&args).is_err());
    }

    #[test]
    fn build_rules_counter() {
        let args = RenameArgs {
            counter_start: Some(1),
            counter_padding: Some(3),
            counter_position: "prefix".into(),
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
            counter_position: "suffix".into(),
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
            counter_position: "replace".into(),
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
    fn build_rules_counter_position_invalid() {
        let args = RenameArgs {
            counter_padding: Some(3),
            counter_position: "invalid".into(),
            ..default_args()
        };
        assert!(build_rules(&args).is_err());
    }

    #[test]
    fn build_rules_multiple_combined() {
        let args = RenameArgs {
            prefix: Some("vacation_".into()),
            case: Some("lower".into()),
            counter_start: Some(1),
            counter_padding: Some(3),
            counter_position: "prefix".into(),
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
    fn build_rules_counter_explicit_defaults() {
        // User explicitly passes --counter-start 1 --counter-padding 0
        // These are the defaults, but counter should still be added
        let args = RenameArgs {
            counter_start: Some(1),
            counter_padding: Some(0),
            counter_position: "prefix".into(),
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
