#[cfg(test)]
mod tests {
    use crate::rules::{FileMetadata, RenameContext, RenameRule};
    use std::path::PathBuf;

    fn make_ctx(stem: &str) -> RenameContext<'static> {
        use std::sync::LazyLock;
        static PARENT: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("/tmp"));
        RenameContext {
            filename: "file.txt",
            stem: stem.to_string(),
            extension: "txt".to_string(),
            parent_dir: &PARENT,
            counter: 0,
            metadata: FileMetadata {
                size: 0,
                modified: None,
                created: None,
            },
            regex_cache: None,
        }
    }

    // --- Basic regex ---

    #[test]
    fn basic_capture_group() {
        let ctx = make_ctx("file_42");
        let re = ctx.get_regex(r"(\d+)").unwrap();
        assert_eq!(
            re.replace_all(&ctx.stem, "img_$1").into_owned(),
            "file_img_42"
        );
    }

    #[test]
    fn replace_literal() {
        let ctx = make_ctx("hello_world");
        let re = ctx.get_regex(r"_").unwrap();
        assert_eq!(re.replace_all(&ctx.stem, "-").into_owned(), "hello-world");
    }

    // --- Multiple occurrences ---

    #[test]
    fn multiple_matches() {
        let ctx = make_ctx("a1b2c3");
        let re = ctx.get_regex(r"\d").unwrap();
        assert_eq!(re.replace_all(&ctx.stem, "X").into_owned(), "aXbXcX");
    }

    // --- No match ---

    #[test]
    fn no_match() {
        let ctx = make_ctx("hello");
        let re = ctx.get_regex(r"\d+").unwrap();
        assert_eq!(re.replace_all(&ctx.stem, "num").into_owned(), "hello");
    }

    // --- Anchors ---

    #[test]
    fn anchored_at_start() {
        let ctx = make_ctx("photo_001");
        let re = ctx.get_regex(r"^photo").unwrap();
        assert_eq!(re.replace_all(&ctx.stem, "img").into_owned(), "img_001");
    }

    #[test]
    fn anchored_at_end() {
        let ctx = make_ctx("file_001");
        let re = ctx.get_regex(r"_\d+$").unwrap();
        assert_eq!(re.replace_all(&ctx.stem, "").into_owned(), "file");
    }

    // --- Complex patterns ---

    #[test]
    fn capture_and_rearrange() {
        let ctx = make_ctx("2024-06-15_photo");
        let re = ctx.get_regex(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
        assert_eq!(
            re.replace_all(&ctx.stem, "$3-$2-$1").into_owned(),
            "15-06-2024_photo"
        );
    }

    #[test]
    fn case_insensitive_pattern() {
        let ctx = make_ctx("Hello_World");
        let re = ctx.get_regex(r"(?i)hello").unwrap();
        assert_eq!(re.replace_all(&ctx.stem, "hi").into_owned(), "hi_World");
    }

    // --- Edge cases ---

    #[test]
    fn empty_replacement() {
        let ctx = make_ctx("file_42_test");
        let re = ctx.get_regex(r"_\d+").unwrap();
        assert_eq!(re.replace_all(&ctx.stem, "").into_owned(), "file_test");
    }

    #[test]
    fn empty_stem() {
        let ctx = make_ctx("");
        let re = ctx.get_regex(r".*").unwrap();
        assert_eq!(
            re.replace_all(&ctx.stem, "replaced").into_owned(),
            "replaced"
        );
    }

    // --- Error cases ---

    #[test]
    fn invalid_regex() {
        let ctx = make_ctx("file");
        assert!(ctx.get_regex(r"[").is_err());
    }

    #[test]
    fn invalid_regex_error_message() {
        let ctx = make_ctx("file");
        let err = ctx.get_regex(r"[").unwrap_err();
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
