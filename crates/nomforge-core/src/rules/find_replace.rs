use crate::error::Result;
use crate::rules::RenameContext;

/// Apply a plain text find & replace on the filename stem.
pub fn apply_find_replace(find: &str, replace: &str, ctx: &RenameContext) -> Result<String> {
    Ok(ctx.stem.replace(find, replace))
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn basic_replace() {
        let ctx = make_ctx("hello world");
        assert_eq!(
            apply_find_replace("world", "rust", &ctx).unwrap(),
            "hello rust"
        );
    }

    #[test]
    fn replace_multiple_occurrences() {
        let ctx = make_ctx("aaa");
        assert_eq!(apply_find_replace("a", "b", &ctx).unwrap(), "bbb");
    }

    #[test]
    fn replace_not_found() {
        let ctx = make_ctx("hello");
        assert_eq!(apply_find_replace("xyz", "abc", &ctx).unwrap(), "hello");
    }

    #[test]
    fn replace_empty_find() {
        let ctx = make_ctx("hello");
        assert_eq!(apply_find_replace("", "x", &ctx).unwrap(), "xhxexlxlxox");
    }

    #[test]
    fn replace_empty_replace() {
        let ctx = make_ctx("hello");
        assert_eq!(apply_find_replace("l", "", &ctx).unwrap(), "heo");
    }

    #[test]
    fn replace_with_special_chars() {
        let ctx = make_ctx("file_001.txt");
        assert_eq!(
            apply_find_replace("_001", "-copy", &ctx).unwrap(),
            "file-copy.txt"
        );
    }

    #[test]
    fn via_enum_variant() {
        use std::sync::LazyLock;
        static PARENT: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("/tmp"));
        let rule = RenameRule::FindReplace {
            find: "DSC".into(),
            replace: "photo".into(),
        };
        let ctx = RenameContext {
            filename: "DSC_001.jpg",
            stem: "DSC_001".into(),
            extension: "jpg".into(),
            parent_dir: &PARENT,
            counter: 0,
            metadata: FileMetadata {
                size: 0,
                modified: None,
                created: None,
            },
            regex_cache: None,
        };
        assert_eq!(rule.apply(&ctx).unwrap(), "photo_001");
    }
}
