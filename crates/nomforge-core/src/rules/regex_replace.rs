#[cfg(test)]
mod tests {
    use crate::rules::{FileMetadata, RegexCache, RenameContext, RenameRule};
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
        let cache = RegexCache::new();
        let rule = RenameRule::RegexReplace {
            pattern: r"(\d+)".into(),
            replacement: "img_$1".into(),
        };
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "file_img_42");
    }

    #[test]
    fn replace_literal() {
        let ctx = make_ctx("hello_world");
        let cache = RegexCache::new();
        let rule = RenameRule::RegexReplace {
            pattern: r"_".into(),
            replacement: "-".into(),
        };
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "hello-world");
    }

    // --- Multiple occurrences ---

    #[test]
    fn multiple_matches() {
        let ctx = make_ctx("a1b2c3");
        let cache = RegexCache::new();
        let rule = RenameRule::RegexReplace {
            pattern: r"\d".into(),
            replacement: "X".into(),
        };
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "aXbXcX");
    }

    // --- No match ---

    #[test]
    fn no_match() {
        let ctx = make_ctx("hello");
        let cache = RegexCache::new();
        let rule = RenameRule::RegexReplace {
            pattern: r"\d+".into(),
            replacement: "num".into(),
        };
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "hello");
    }

    // --- Anchors ---

    #[test]
    fn anchored_at_start() {
        let ctx = make_ctx("photo_001");
        let cache = RegexCache::new();
        let rule = RenameRule::RegexReplace {
            pattern: r"^photo".into(),
            replacement: "img".into(),
        };
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "img_001");
    }

    #[test]
    fn anchored_at_end() {
        let ctx = make_ctx("file_001");
        let cache = RegexCache::new();
        let rule = RenameRule::RegexReplace {
            pattern: r"_\d+$".into(),
            replacement: "".into(),
        };
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "file");
    }

    // --- Complex patterns ---

    #[test]
    fn capture_and_rearrange() {
        let ctx = make_ctx("2024-06-15_photo");
        let cache = RegexCache::new();
        let rule = RenameRule::RegexReplace {
            pattern: r"(\d{4})-(\d{2})-(\d{2})".into(),
            replacement: "$3-$2-$1".into(),
        };
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "15-06-2024_photo");
    }

    #[test]
    fn case_insensitive_pattern() {
        let ctx = make_ctx("Hello_World");
        let cache = RegexCache::new();
        let rule = RenameRule::RegexReplace {
            pattern: r"(?i)hello".into(),
            replacement: "hi".into(),
        };
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "hi_World");
    }

    // --- Edge cases ---

    #[test]
    fn empty_replacement() {
        let ctx = make_ctx("file_42_test");
        let cache = RegexCache::new();
        let rule = RenameRule::RegexReplace {
            pattern: r"_\d+".into(),
            replacement: "".into(),
        };
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "file_test");
    }

    // --- Error cases ---

    #[test]
    fn invalid_regex() {
        let ctx = make_ctx("file");
        let cache = RegexCache::new();
        let rule = RenameRule::RegexReplace {
            pattern: r"[".into(),
            replacement: "x".into(),
        };
        assert!(rule.apply(&ctx, &cache).is_err());
    }

    #[test]
    fn invalid_regex_error_message() {
        let ctx = make_ctx("file");
        let cache = RegexCache::new();
        let rule = RenameRule::RegexReplace {
            pattern: r"[".into(),
            replacement: "x".into(),
        };
        let err = rule.apply(&ctx, &cache).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("["));
        assert!(msg.contains("Invalid regex pattern"));
    }

    // --- Cache reuse ---

    #[test]
    fn cache_reuses_compiled_regex() {
        let ctx = make_ctx("file_42");
        let cache = RegexCache::new();
        let rule = RenameRule::RegexReplace {
            pattern: r"(\d+)".into(),
            replacement: "num_$1".into(),
        };
        // First use compiles and caches
        let result1 = rule.apply(&ctx, &cache).unwrap();
        // Second use reuses cached regex
        let result2 = rule.apply(&ctx, &cache).unwrap();
        assert_eq!(result1, result2);
        assert_eq!(result1, "file_num_42");
    }
}
