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
    use crate::rules::{FileMetadata, RenameContext, RenameRule};
    use std::path::PathBuf;

    fn make_ctx(stem: &str, counter: usize) -> RenameContext<'static> {
        use std::sync::LazyLock;
        static PARENT: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("/tmp"));
        RenameContext {
            filename: "file.txt",
            stem: stem.to_string(),
            extension: "txt".to_string(),
            parent_dir: &PARENT,
            counter,
            metadata: FileMetadata {
                size: 0,
                modified: None,
                created: None,
            },
            regex_cache: None,
        }
    }

    // --- Prefix position ---

    #[test]
    fn prefix_basic() {
        let ctx = make_ctx("photo", 0);
        assert_eq!(
            apply_counter(1, 3, SeqPosition::Prefix, &ctx).unwrap(),
            "001photo"
        );
    }

    #[test]
    fn prefix_second_file() {
        let ctx = make_ctx("photo", 1);
        assert_eq!(
            apply_counter(1, 3, SeqPosition::Prefix, &ctx).unwrap(),
            "002photo"
        );
    }

    #[test]
    fn prefix_no_padding() {
        let ctx = make_ctx("photo", 0);
        assert_eq!(
            apply_counter(1, 0, SeqPosition::Prefix, &ctx).unwrap(),
            "1photo"
        );
    }

    #[test]
    fn prefix_wide_padding() {
        let ctx = make_ctx("photo", 0);
        assert_eq!(
            apply_counter(1, 6, SeqPosition::Prefix, &ctx).unwrap(),
            "000001photo"
        );
    }

    // --- Suffix position ---

    #[test]
    fn suffix_basic() {
        let ctx = make_ctx("photo", 0);
        assert_eq!(
            apply_counter(1, 3, SeqPosition::Suffix, &ctx).unwrap(),
            "photo001"
        );
    }

    #[test]
    fn suffix_second_file() {
        let ctx = make_ctx("photo", 1);
        assert_eq!(
            apply_counter(10, 2, SeqPosition::Suffix, &ctx).unwrap(),
            "photo11"
        );
    }

    // --- ReplaceStem position ---

    #[test]
    fn replace_stem_basic() {
        let ctx = make_ctx("photo", 0);
        assert_eq!(
            apply_counter(1, 4, SeqPosition::ReplaceStem, &ctx).unwrap(),
            "0001"
        );
    }

    #[test]
    fn replace_stem_second_file() {
        let ctx = make_ctx("photo", 1);
        assert_eq!(
            apply_counter(5, 4, SeqPosition::ReplaceStem, &ctx).unwrap(),
            "0006"
        );
    }

    // --- Edge cases ---

    #[test]
    fn start_at_zero() {
        let ctx = make_ctx("file", 0);
        assert_eq!(
            apply_counter(0, 2, SeqPosition::Prefix, &ctx).unwrap(),
            "00file"
        );
    }

    #[test]
    fn large_counter_offset() {
        let ctx = make_ctx("file", 99);
        assert_eq!(
            apply_counter(1, 3, SeqPosition::Suffix, &ctx).unwrap(),
            "file100"
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
        let ctx = make_ctx("photo", 0);
        assert_eq!(rule.apply(&ctx).unwrap(), "001photo");
    }

    #[test]
    fn via_enum_suffix() {
        let rule = RenameRule::NumberSequence {
            start: 10,
            padding: 2,
            position: SeqPosition::Suffix,
        };
        let ctx = make_ctx("photo", 0);
        assert_eq!(rule.apply(&ctx).unwrap(), "photo10");
    }

    #[test]
    fn via_enum_replace_stem() {
        let rule = RenameRule::NumberSequence {
            start: 5,
            padding: 4,
            position: SeqPosition::ReplaceStem,
        };
        let ctx = make_ctx("photo", 0);
        assert_eq!(rule.apply(&ctx).unwrap(), "0005");
    }
}
