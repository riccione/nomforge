use crate::error::Result;
use crate::rules::RenameContext;

/// Prepend text to the filename stem.
pub fn apply_prefix(prefix: &str, ctx: &RenameContext) -> Result<String> {
    Ok(format!("{}{}", prefix, ctx.stem))
}

/// Append text to the filename stem.
pub fn apply_suffix(suffix: &str, ctx: &RenameContext) -> Result<String> {
    Ok(format!("{}{}", ctx.stem, suffix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{FileMetadata, RenameContext, RenameRule};
    use std::path::PathBuf;

    fn make_ctx(stem: &str) -> RenameContext {
        RenameContext {
            filename: format!("{}.txt", stem),
            stem: stem.to_string(),
            extension: "txt".to_string(),
            parent_dir: PathBuf::from("/tmp"),
            counter: 0,
            metadata: FileMetadata {
                size: 0,
                modified: None,
                created: None,
            },
        }
    }

    // --- Prefix tests ---

    #[test]
    fn prefix_basic() {
        let ctx = make_ctx("photo");
        assert_eq!(apply_prefix("vacation_", &ctx).unwrap(), "vacation_photo");
    }

    #[test]
    fn prefix_empty() {
        let ctx = make_ctx("photo");
        assert_eq!(apply_prefix("", &ctx).unwrap(), "photo");
    }

    #[test]
    fn prefix_special_chars() {
        let ctx = make_ctx("doc");
        assert_eq!(apply_prefix("[2024]_", &ctx).unwrap(), "[2024]_doc");
    }

    #[test]
    fn prefix_via_enum() {
        let rule = RenameRule::Prefix("vacation_".into());
        let ctx = make_ctx("beach");
        assert_eq!(rule.apply(&ctx).unwrap(), "vacation_beach");
    }

    // --- Suffix tests ---

    #[test]
    fn suffix_basic() {
        let ctx = make_ctx("report");
        assert_eq!(apply_suffix("_final", &ctx).unwrap(), "report_final");
    }

    #[test]
    fn suffix_empty() {
        let ctx = make_ctx("report");
        assert_eq!(apply_suffix("", &ctx).unwrap(), "report");
    }

    #[test]
    fn suffix_special_chars() {
        let ctx = make_ctx("photo");
        assert_eq!(apply_suffix(" (copy)", &ctx).unwrap(), "photo (copy)");
    }

    #[test]
    fn suffix_via_enum() {
        let rule = RenameRule::Suffix("_final".into());
        let ctx = make_ctx("report");
        assert_eq!(rule.apply(&ctx).unwrap(), "report_final");
    }

    // --- Combined prefix + suffix ---

    #[test]
    fn prefix_and_suffix_together() {
        let prefix_rule = RenameRule::Prefix("pre_".into());
        let suffix_rule = RenameRule::Suffix("_suf".into());
        let ctx = make_ctx("file");

        let step1 = prefix_rule.apply(&ctx).unwrap();
        assert_eq!(step1, "pre_file");

        let ctx2 = RenameContext { stem: step1, ..ctx };
        let step2 = suffix_rule.apply(&ctx2).unwrap();
        assert_eq!(step2, "pre_file_suf");
    }
}
