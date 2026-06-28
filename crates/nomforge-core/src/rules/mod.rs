mod case_transform;
mod counter;
mod extension;
mod find_replace;
mod prefix_suffix;
mod regex_replace;
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
            Self::ChangeExtension { new_ext } => {
                extension::apply_extension(&ctx.extension, new_ext)?;
                Ok(ctx.stem.clone())
            }
            Self::RegexReplace {
                pattern,
                replacement,
            } => regex_replace::apply_regex_replace(pattern, replacement, ctx),
        }
    }
}
