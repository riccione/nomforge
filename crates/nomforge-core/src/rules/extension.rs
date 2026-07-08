use crate::error::{NomforgeError, Result};

/// Validate that an extension string is safe.
///
/// Rejects extensions containing:
/// - Path separators (`/` or `\`)
/// - Parent directory reference (`..`)
/// - Null bytes
fn validate_extension(ext: &str) -> Result<()> {
    if ext.contains('/') || ext.contains('\\') {
        return Err(NomforgeError::InvalidExtension {
            ext: ext.to_string(),
            reason: "contains path separator".to_string(),
        });
    }
    if ext == ".." || ext.contains("/..") || ext.contains("\\..") || ext.contains("..\\") {
        return Err(NomforgeError::InvalidExtension {
            ext: ext.to_string(),
            reason: "contains parent directory reference".to_string(),
        });
    }
    if ext.contains('\0') {
        return Err(NomforgeError::InvalidExtension {
            ext: ext.to_string(),
            reason: "contains null byte".to_string(),
        });
    }
    Ok(())
}

/// Determine the new file extension based on the rule.
///
/// - `current_ext`: the current extension (without dot)
/// - `new_ext`: `None` to keep current, `Some("")` to remove, `Some("png")` to change
///
/// Returns the new extension (without dot).
pub fn apply_extension(current_ext: &str, new_ext: &Option<String>) -> Result<String> {
    match new_ext {
        None => Ok(current_ext.to_string()),
        Some(ext) if ext.is_empty() => Ok(String::new()),
        Some(ext) => {
            validate_extension(ext)?;
            Ok(ext.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{FileMetadata, RegexCache, RenameContext, RenameRule};
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
        let cache = RegexCache::new();
        // The apply() method returns the stem unchanged for ChangeExtension
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "file");
    }

    #[test]
    fn via_enum_keep_returns_stem_unchanged() {
        let rule = RenameRule::ChangeExtension { new_ext: None };
        let ctx = make_ctx("jpg");
        let cache = RegexCache::new();
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "file");
    }

    // --- Validation tests ---

    #[test]
    fn reject_path_separator_forward_slash() {
        let result = apply_extension("txt", &Some("../etc/passwd".into()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("path separator"));
    }

    #[test]
    fn reject_path_separator_backslash() {
        let result = apply_extension("txt", &Some("..\\etc\\passwd".into()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("path separator"));
    }

    #[test]
    fn reject_parent_directory_reference() {
        let result = apply_extension("txt", &Some("..".into()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("parent directory"));
    }

    #[test]
    fn reject_null_byte() {
        let result = apply_extension("txt", &Some("png\0jpg".into()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("null byte"));
    }

    #[test]
    fn allow_valid_extensions() {
        assert!(apply_extension("txt", &Some("png".into())).is_ok());
        assert!(apply_extension("txt", &Some("jpg".into())).is_ok());
        assert!(apply_extension("txt", &Some("md".into())).is_ok());
        assert!(apply_extension("txt", &Some("".into())).is_ok());
    }
}
