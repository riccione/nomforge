mod case_transform;
mod counter;
mod extension;
mod find_replace;
mod prefix_suffix;
mod regex_replace;
mod remove_text;

use std::collections::HashMap;
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::{NomforgeError, Result};

/// Context passed to each rename rule during transformation.
#[derive(Debug, Clone)]
pub struct RenameContext {
    /// Filename without extension (e.g. "photo_001")
    pub stem: String,
    /// Extension without dot (e.g. "jpg")
    pub extension: String,
    /// 0-based index of this file in the batch
    pub counter: usize,
}

/// Wrapper for pre-compiled regex cache.
pub struct RegexCache {
    cache: HashMap<String, OnceLock<Regex>>,
}

/// Compile `pattern`, mapping regex errors to [`NomforgeError::InvalidRegex`].
fn compile_regex(pattern: &str) -> Result<Regex> {
    Regex::new(pattern).map_err(|e| NomforgeError::InvalidRegex {
        pattern: pattern.to_string(),
        reason: e.to_string(),
    })
}

impl RegexCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Pre-compile patterns from rules.
    pub fn from_rules(rules: &[RenameRule]) -> Self {
        let mut cache = HashMap::new();
        for rule in rules {
            if let RenameRule::RegexReplace { pattern, .. } = rule {
                cache.entry(pattern.clone()).or_insert_with(OnceLock::new);
            }
        }
        Self { cache }
    }

    /// Get or compile a regex pattern.
    pub fn get_regex(&self, pattern: &str) -> Result<Regex> {
        if let Some(lock) = self.cache.get(pattern) {
            if let Some(re) = lock.get() {
                return Ok(re.clone());
            }
            // Not compiled yet: compile once, then store race-safely. If
            // another thread won the race, `get_or_init` returns their
            // instance and ours is dropped.
            let compiled = compile_regex(pattern)?;
            return Ok(lock.get_or_init(|| compiled).clone());
        }
        // Pattern not in cache, compile directly (for ad-hoc usage)
        compile_regex(pattern)
    }
}

impl Default for RegexCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Case transformation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Case {
    Upper,
    Lower,
    /// Capitalize each word; remaining characters are lowercased.
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
///
/// Serialized as `{"type": "<Variant>", "value": <payload>}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
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
    pub fn apply(&self, ctx: &RenameContext, cache: &RegexCache) -> Result<String> {
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
                extension::apply_extension(&ctx.extension, new_ext.as_deref())?;
                Ok(ctx.stem.clone())
            }
            Self::RegexReplace {
                pattern,
                replacement,
            } => {
                let re = cache.get_regex(pattern)?;
                Ok(re.replace_all(&ctx.stem, replacement).into_owned())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn regex_rule(pattern: &str) -> RenameRule {
        RenameRule::RegexReplace {
            pattern: pattern.to_string(),
            replacement: String::new(),
        }
    }

    #[test]
    fn get_regex_caches_compiled_pattern() {
        let cache = RegexCache::from_rules(&[regex_rule(r"\d+")]);

        let first = cache.get_regex(r"\d+").unwrap();
        let second = cache.get_regex(r"\d+").unwrap();

        assert_eq!(first.as_str(), r"\d+");
        assert_eq!(second.as_str(), r"\d+");
        // The OnceLock for the pre-registered pattern is now initialized.
        assert!(cache.cache[r"\d+"].get().is_some());
    }

    #[test]
    fn get_regex_invalid_pattern_in_cache_errors() {
        let cache = RegexCache::from_rules(&[regex_rule("(")]);

        let err = cache.get_regex("(").unwrap_err();
        assert!(err.to_string().contains("Invalid regex"), "err: {err}");

        // Failed compilations are not cached: the lock stays empty.
        assert!(cache.cache["("].get().is_none());
        assert!(cache.get_regex("(").is_err());
    }

    #[test]
    fn get_regex_uncached_pattern_compiles() {
        let cache = RegexCache::new();

        let re = cache.get_regex(r"\w+").unwrap();
        assert_eq!(re.as_str(), r"\w+");
    }

    #[test]
    fn get_regex_concurrent_first_use() {
        let cache = RegexCache::from_rules(&[regex_rule(r"\d+")]);

        std::thread::scope(|s| {
            for _ in 0..8 {
                s.spawn(|| {
                    let re = cache.get_regex(r"\d+").unwrap();
                    assert_eq!(re.as_str(), r"\d+");
                });
            }
        });
    }

    // --- Serde round-trip ---

    #[test]
    fn rule_serde_roundtrip_all_variants() {
        let rules = vec![
            RenameRule::FindReplace {
                find: "old".into(),
                replace: "new".into(),
            },
            RenameRule::Prefix("pre_".into()),
            RenameRule::Suffix("_suf".into()),
            RenameRule::RemoveText("copy".into()),
            RenameRule::CaseTransform(Case::Title),
            RenameRule::NumberSequence {
                start: 5,
                padding: 3,
                position: SeqPosition::Suffix,
            },
            RenameRule::ChangeExtension {
                new_ext: Some("png".into()),
            },
            RenameRule::ChangeExtension { new_ext: None },
            RenameRule::RegexReplace {
                pattern: r"\d+".into(),
                replacement: "n".into(),
            },
        ];

        for rule in &rules {
            let json =
                serde_json::to_string(rule).unwrap_or_else(|e| panic!("serialize {rule:?}: {e}"));
            let back: RenameRule =
                serde_json::from_str(&json).unwrap_or_else(|e| panic!("deserialize {json}: {e}"));
            assert_eq!(&back, rule, "json: {json}");
        }
    }

    #[test]
    fn rule_serde_uses_adjacent_tagging() {
        let json = serde_json::to_string(&RenameRule::Prefix("pre_".into())).unwrap();
        assert!(json.contains(r#""type":"Prefix""#), "json: {json}");
        assert!(json.contains(r#""value":"pre_""#), "json: {json}");
    }
}
