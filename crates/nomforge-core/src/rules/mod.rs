mod case_transform;
mod counter;
mod find_replace;
mod prefix_suffix;
mod remove_text;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Metadata about a file being renamed.
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: Option<std::time::SystemTime>,
    pub created: Option<std::time::SystemTime>,
}

/// Context passed to each rename rule during transformation.
#[derive(Debug, Clone)]
pub struct RenameContext {
    /// Full original filename (e.g. "photo_001.jpg")
    pub filename: String,
    /// Filename without extension (e.g. "photo_001")
    pub stem: String,
    /// Extension without dot (e.g. "jpg")
    pub extension: String,
    /// Parent directory path
    pub parent_dir: PathBuf,
    /// 0-based index of this file in the batch
    pub counter: usize,
    /// File metadata
    pub metadata: FileMetadata,
}

/// Case transformation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Case {
    Upper,
    Lower,
    Title,
}

/// Where to insert the counter in the filename.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeqPosition {
    Prefix,
    Suffix,
    ReplaceStem,
}

/// A single rename rule applied to a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RenameRule {
    /// Plain text find & replace (no regex).
    FindReplace { find: String, replace: String },
    /// Prepend text to filename stem.
    Prefix(String),
    /// Append text to filename stem.
    Suffix(String),
    /// Delete all occurrences of a substring from the filename.
    RemoveText(String),
    /// Case transformation.
    CaseTransform(Case),
    /// Insert/replace with a sequential number.
    NumberSequence {
        start: usize,
        padding: usize,
        position: SeqPosition,
    },
    /// Keep or change the file extension.
    ChangeExtension { new_ext: Option<String> },
    /// Regex-based find & replace.
    RegexReplace {
        pattern: String,
        replacement: String,
    },
}

impl RenameRule {
    /// Apply this rule to the given context, returning the transformed stem.
    ///
    /// Rules operate on the stem (filename without extension) unless otherwise noted.
    /// The extension is preserved separately and reattached by the engine.
    pub fn apply(&self, ctx: &RenameContext) -> Result<String> {
        match self {
            Self::FindReplace { find, replace } => {
                find_replace::apply_find_replace(find, replace, ctx)
            }
            Self::Prefix(p) => prefix_suffix::apply_prefix(p, ctx),
            Self::Suffix(s) => prefix_suffix::apply_suffix(s, ctx),
            Self::RemoveText(text) => remove_text::apply_remove_text(text, ctx),
            Self::CaseTransform(case) => case_transform::apply_case_transform(&ctx.stem, *case),
            Self::NumberSequence {
                start,
                padding,
                position,
            } => counter::apply_counter(*start, *padding, *position, ctx),
            Self::ChangeExtension { new_ext: _ } => {
                // Extension changes are handled by the engine, not on the stem.
                // Return stem unchanged.
                Ok(ctx.stem.clone())
            }
            Self::RegexReplace {
                pattern,
                replacement,
            } => {
                let re = regex::Regex::new(pattern).map_err(|e| {
                    crate::error::NomforgeError::InvalidRegex {
                        pattern: pattern.clone(),
                        reason: e.to_string(),
                    }
                })?;
                Ok(re.replace(&ctx.stem, replacement.as_str()).into_owned())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(stem: &str, ext: &str) -> RenameContext {
        RenameContext {
            filename: format!("{}.{}", stem, ext),
            stem: stem.to_string(),
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

    #[test]
    fn regex_replace() {
        let rule = RenameRule::RegexReplace {
            pattern: r"(\d+)".into(),
            replacement: "img_$1".into(),
        };
        let ctx = make_ctx("file_42", "jpg");
        assert_eq!(rule.apply(&ctx).unwrap(), "file_img_42");
    }

    #[test]
    fn regex_invalid_pattern() {
        let rule = RenameRule::RegexReplace {
            pattern: r"[".into(),
            replacement: "x".into(),
        };
        let ctx = make_ctx("file", "txt");
        assert!(rule.apply(&ctx).is_err());
    }
}
