use crate::error::Result;
use crate::rules::RenameContext;

/// Delete all occurrences of a substring from the filename stem.
pub fn apply_remove_text(text: &str, ctx: &RenameContext) -> Result<String> {
    Ok(ctx.stem.replace(text, ""))
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn remove_basic() {
        let ctx = make_ctx("hello_world");
        assert_eq!(apply_remove_text("_world", &ctx).unwrap(), "hello");
    }

    #[test]
    fn remove_multiple_occurrences() {
        let ctx = make_ctx("aaa");
        assert_eq!(apply_remove_text("a", &ctx).unwrap(), "");
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
    fn via_enum_variant() {
        let rule = RenameRule::RemoveText("_copy".into());
        let ctx = make_ctx("file_copy");
        let cache = RegexCache::new();
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "file");
    }
}
