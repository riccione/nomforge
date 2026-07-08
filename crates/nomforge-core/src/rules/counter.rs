use crate::error::Result;
use crate::rules::{RenameContext, SeqPosition};

/// Apply a sequential number to the filename stem.
///
/// - `start`: the first number in the sequence (0-based offset is added internally)
/// - `padding`: minimum number of digits (zero-padded)
/// - `position`: where to place the number relative to the stem
/// - `counter`: the 0-based index of this file in the batch
pub fn apply_counter(
    start: usize,
    padding: usize,
    position: SeqPosition,
    ctx: &RenameContext,
) -> Result<String> {
    let num = start + ctx.counter;
    let padded = format!("{:0>width$}", num, width = padding);
    Ok(match position {
        SeqPosition::Prefix => format!("{}{}", padded, ctx.stem),
        SeqPosition::Suffix => format!("{}{}", ctx.stem, padded),
        SeqPosition::ReplaceStem => padded,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{FileMetadata, RegexCache, RenameContext, RenameRule};
    use std::path::PathBuf;

    fn make_ctx(stem: &str, counter: usize) -> RenameContext {
        RenameContext {
            filename: format!("{}.txt", stem),
            stem: stem.to_string(),
            extension: "txt".to_string(),
            parent_dir: PathBuf::from("/tmp"),
            counter,
            metadata: FileMetadata {
                size: 0,
                modified: None,
                created: None,
            },
        }
    }

    // --- Prefix ---

    #[test]
    fn counter_prefix_basic() {
        let ctx = make_ctx("photo", 0);
        assert_eq!(
            apply_counter(1, 3, SeqPosition::Prefix, &ctx).unwrap(),
            "001photo"
        );
    }

    #[test]
    fn counter_prefix_no_padding() {
        let ctx = make_ctx("photo", 5);
        assert_eq!(
            apply_counter(0, 0, SeqPosition::Prefix, &ctx).unwrap(),
            "5photo"
        );
    }

    // --- Suffix ---

    #[test]
    fn counter_suffix_basic() {
        let ctx = make_ctx("photo", 0);
        assert_eq!(
            apply_counter(1, 3, SeqPosition::Suffix, &ctx).unwrap(),
            "photo001"
        );
    }

    // --- ReplaceStem ---

    #[test]
    fn counter_replace_stem() {
        let ctx = make_ctx("photo", 0);
        assert_eq!(
            apply_counter(1, 3, SeqPosition::ReplaceStem, &ctx).unwrap(),
            "001"
        );
    }

    // --- Via enum variant ---

    #[test]
    fn via_enum_prefix() {
        let rule = RenameRule::NumberSequence {
            start: 1,
            padding: 3,
            position: SeqPosition::Prefix,
        };
        let ctx = make_ctx("file", 0);
        let cache = RegexCache::new();
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "001file");
    }

    #[test]
    fn via_enum_suffix() {
        let rule = RenameRule::NumberSequence {
            start: 1,
            padding: 3,
            position: SeqPosition::Suffix,
        };
        let ctx = make_ctx("file", 0);
        let cache = RegexCache::new();
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "file001");
    }

    #[test]
    fn via_enum_replace_stem() {
        let rule = RenameRule::NumberSequence {
            start: 1,
            padding: 3,
            position: SeqPosition::ReplaceStem,
        };
        let ctx = make_ctx("file", 0);
        let cache = RegexCache::new();
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "001");
    }

    // --- Counter offset ---

    #[test]
    fn counter_offset() {
        let ctx = make_ctx("file", 5);
        assert_eq!(
            apply_counter(10, 3, SeqPosition::Prefix, &ctx).unwrap(),
            "015file"
        );
    }
}
