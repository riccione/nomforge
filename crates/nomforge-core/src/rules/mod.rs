mod case_transform;
mod counter;
mod extension;
mod find_replace;
mod prefix_suffix;
mod regex_replace;
mod remove_text;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::{NomforgeError, Result};

/// Metadata about a file being renamed.
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: Option<std::time::SystemTime>,
    pub created: Option<std::time::SystemTime>,
}

/// Context passed to each rename rule during transformation.
pub struct RenameContext<'a> {
    /// Full original filename (e.g. "photo_001.jpg")
    pub filename: &'a str,
    /// Filename without extension (e.g. "photo_001")
    pub stem: String,
    /// Extension without dot (e.g. "jpg")
    pub extension: String,
    /// Parent directory path
    pub parent_dir: &'a PathBuf,
    /// 0-based index of this file in the batch
    pub counter: usize,
    /// File metadata
    pub metadata: FileMetadata,
    /// Cache of compiled regexes (optional, for reuse across rules)
    pub regex_cache: Option<&'a HashMap<String, OnceLock<Regex>>>,
}

impl<'a> RenameContext<'a> {
    /// Get or compile a regex pattern, using the cache if available.
    pub fn get_regex(&self, pattern: &str) -> Result<Regex> {
        if let Some(cache) = self.regex_cache {
            let lock = cache
                .get(pattern)
                .ok_or_else(|| NomforgeError::InvalidRegex {
                    pattern: pattern.to_string(),
                    reason: "pattern not found in cache".to_string(),
                })?;
            if let Some(re) = lock.get() {
                return Ok(re.clone());
            }
            // Compile and store the regex
            let re = Regex::new(pattern).map_err(|e| NomforgeError::InvalidRegex {
                pattern: pattern.to_string(),
                reason: e.to_string(),
            })?;
            let _ = lock.set(re);
            return Ok(lock.get().unwrap().clone());
        }
        // No cache available, compile directly
        Regex::new(pattern).map_err(|e| NomforgeError::InvalidRegex {
            pattern: pattern.to_string(),
            reason: e.to_string(),
        })
    }
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
            } => {
                let re = ctx.get_regex(pattern)?;
                Ok(re.replace_all(&ctx.stem, replacement).into_owned())
            }
        }
    }
}
