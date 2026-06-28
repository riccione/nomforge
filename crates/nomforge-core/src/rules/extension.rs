use crate::error::Result;

/// Determine the new file extension based on the rule.
///
/// - `current_ext`: the current extension (without dot)
/// - `new_ext`: `None` to keep current, `Some("")` to remove, `Some("png")` to change
///
/// Returns the new extension (without dot).
pub fn apply_extension(current_ext: &str, new_ext: &Option<String>) -> Result<String> {
    Ok(match new_ext {
        None => current_ext.to_string(),
        Some(ext) if ext.is_empty() => String::new(),
        Some(ext) => ext.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{FileMetadata, RenameContext, RenameRule};
    use std::path::PathBuf;

    fn make_ctx(ext: &str) -> RenameContext {
        RenameContext {
            filename: format!("file.{}", ext),
            stem: "file".to_string(),
            extension: ext.to_string(),
            parent_dir: PathBuf::from("/tmp"),
            counter: 0,
            metadata: FileMetadata {
                size: 0,
                modified: None,
                created: None,
            },
        }
    }

    // --- Keep current extension ---

    #[test]
    fn keep_current() {
        let ctx = make_ctx("jpg");
        assert_eq!(apply_extension(&ctx.extension, &None).unwrap(), "jpg");
    }

    // --- Change extension ---

    #[test]
    fn change_to_png() {
        let ctx = make_ctx("jpg");
        assert_eq!(
            apply_extension(&ctx.extension, &Some("png".into())).unwrap(),
            "png"
        );
    }

    #[test]
    fn change_to_pdf() {
        let ctx = make_ctx("docx");
        assert_eq!(
            apply_extension(&ctx.extension, &Some("pdf".into())).unwrap(),
            "pdf"
        );
    }

    // --- Remove extension ---

    #[test]
    fn remove_extension() {
        let ctx = make_ctx("txt");
        assert_eq!(
            apply_extension(&ctx.extension, &Some("".into())).unwrap(),
            ""
        );
    }

    // --- Same extension ---

    #[test]
    fn same_extension() {
        let ctx = make_ctx("jpg");
        assert_eq!(
            apply_extension(&ctx.extension, &Some("jpg".into())).unwrap(),
            "jpg"
        );
    }

    // --- Via enum variant (stem unchanged) ---

    #[test]
    fn via_enum_returns_stem_unchanged() {
        let rule = RenameRule::ChangeExtension {
            new_ext: Some("png".into()),
        };
        let ctx = make_ctx("jpg");
        // The apply() method returns the stem unchanged for ChangeExtension
        assert_eq!(rule.apply(&ctx).unwrap(), "file");
    }

    #[test]
    fn via_enum_keep_returns_stem_unchanged() {
        let rule = RenameRule::ChangeExtension { new_ext: None };
        let ctx = make_ctx("jpg");
        assert_eq!(rule.apply(&ctx).unwrap(), "file");
    }
}
