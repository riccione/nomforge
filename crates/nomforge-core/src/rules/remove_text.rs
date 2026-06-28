use crate::error::Result;
use crate::rules::RenameContext;

/// Delete all occurrences of a substring from the filename stem.
pub fn apply_remove_text(text: &str, ctx: &RenameContext) -> Result<String> {
    Ok(ctx.stem.replace(text, ""))
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

    #[test]
    fn remove_space() {
        let ctx = make_ctx("hello world");
        assert_eq!(apply_remove_text(" ", &ctx).unwrap(), "helloworld");
    }

    #[test]
    fn remove_multiple_occurrences() {
        let ctx = make_ctx("a-b-c-d");
        assert_eq!(apply_remove_text("-", &ctx).unwrap(), "abcd");
    }

    #[test]
    fn remove_not_found() {
        let ctx = make_ctx("hello");
        assert_eq!(apply_remove_text("xyz", &ctx).unwrap(), "hello");
    }

    #[test]
    fn remove_empty_text() {
        let ctx = make_ctx("hello");
        assert_eq!(apply_remove_text("", &ctx).unwrap(), "hello");
    }

    #[test]
    fn remove_entire_stem() {
        let ctx = make_ctx("hello");
        assert_eq!(apply_remove_text("hello", &ctx).unwrap(), "");
    }

    #[test]
    fn remove_substring() {
        let ctx = make_ctx("file_copy_final.txt");
        assert_eq!(apply_remove_text("_copy", &ctx).unwrap(), "file_final.txt");
    }

    #[test]
    fn remove_special_chars() {
        let ctx = make_ctx("[photo]");
        assert_eq!(apply_remove_text("[", &ctx).unwrap(), "photo]");
        assert_eq!(apply_remove_text("]", &ctx).unwrap(), "[photo");
    }

    #[test]
    fn via_enum_variant() {
        let rule = RenameRule::RemoveText(" ".into());
        let ctx = make_ctx("hello world");
        assert_eq!(rule.apply(&ctx).unwrap(), "helloworld");
    }
}
