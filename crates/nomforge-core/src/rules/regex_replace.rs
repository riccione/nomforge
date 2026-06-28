use crate::error::{NomforgeError, Result};
use crate::rules::RenameContext;

/// Apply a regex find & replace on the filename stem.
pub fn apply_regex_replace(
    pattern: &str,
    replacement: &str,
    ctx: &RenameContext,
) -> Result<String> {
    let re = regex::Regex::new(pattern).map_err(|e| NomforgeError::InvalidRegex {
        pattern: pattern.to_string(),
        reason: e.to_string(),
    })?;
    Ok(re.replace_all(&ctx.stem, replacement).into_owned())
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

    // --- Basic regex ---

    #[test]
    fn basic_capture_group() {
        let ctx = make_ctx("file_42");
        assert_eq!(
            apply_regex_replace(r"(\d+)", "img_$1", &ctx).unwrap(),
            "file_img_42"
        );
    }

    #[test]
    fn replace_literal() {
        let ctx = make_ctx("hello_world");
        assert_eq!(apply_regex_replace(r"_", "-", &ctx).unwrap(), "hello-world");
    }

    // --- Multiple occurrences ---

    #[test]
    fn multiple_matches() {
        let ctx = make_ctx("a1b2c3");
        assert_eq!(apply_regex_replace(r"\d", "X", &ctx).unwrap(), "aXbXcX");
    }

    // --- No match ---

    #[test]
    fn no_match() {
        let ctx = make_ctx("hello");
        assert_eq!(apply_regex_replace(r"\d+", "num", &ctx).unwrap(), "hello");
    }

    // --- Anchors ---

    #[test]
    fn anchored_at_start() {
        let ctx = make_ctx("photo_001");
        assert_eq!(
            apply_regex_replace(r"^photo", "img", &ctx).unwrap(),
            "img_001"
        );
    }

    #[test]
    fn anchored_at_end() {
        let ctx = make_ctx("file_001");
        assert_eq!(apply_regex_replace(r"_\d+$", "", &ctx).unwrap(), "file");
    }

    // --- Complex patterns ---

    #[test]
    fn capture_and_rearrange() {
        let ctx = make_ctx("2024-06-15_photo");
        assert_eq!(
            apply_regex_replace(r"(\d{4})-(\d{2})-(\d{2})", "$3-$2-$1", &ctx).unwrap(),
            "15-06-2024_photo"
        );
    }

    #[test]
    fn case_insensitive_pattern() {
        let ctx = make_ctx("Hello_World");
        assert_eq!(
            apply_regex_replace(r"(?i)hello", "hi", &ctx).unwrap(),
            "hi_World"
        );
    }

    // --- Edge cases ---

    #[test]
    fn empty_replacement() {
        let ctx = make_ctx("file_42_test");
        assert_eq!(apply_regex_replace(r"_\d+", "", &ctx).unwrap(), "file_test");
    }

    #[test]
    fn empty_stem() {
        let ctx = make_ctx("");
        assert_eq!(
            apply_regex_replace(r".*", "replaced", &ctx).unwrap(),
            "replaced"
        );
    }

    // --- Error cases ---

    #[test]
    fn invalid_regex() {
        let ctx = make_ctx("file");
        assert!(apply_regex_replace(r"[", "x", &ctx).is_err());
    }

    #[test]
    fn invalid_regex_error_message() {
        let ctx = make_ctx("file");
        let err = apply_regex_replace(r"[", "x", &ctx).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("["));
        assert!(msg.contains("Invalid regex pattern"));
    }

    // --- Via enum variant ---

    #[test]
    fn via_enum_basic() {
        let rule = RenameRule::RegexReplace {
            pattern: r"(\d+)".into(),
            replacement: "img_$1".into(),
        };
        let ctx = make_ctx("file_42");
        assert_eq!(rule.apply(&ctx).unwrap(), "file_img_42");
    }

    #[test]
    fn via_enum_invalid() {
        let rule = RenameRule::RegexReplace {
            pattern: r"[".into(),
            replacement: "x".into(),
        };
        let ctx = make_ctx("file");
        assert!(rule.apply(&ctx).is_err());
    }
}
