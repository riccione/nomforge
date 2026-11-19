use std::path::{Path, PathBuf};

/// Maximum filename length in bytes (conservative limit across OSes).
///
/// - Linux (ext4/XFS/Btrfs): 255 bytes per component
/// - macOS (APFS): 255 characters (NFD decomposed)
/// - Windows (NTFS): 255 UTF-16 code units
const MAX_FILENAME_BYTES: usize = 255;

/// Truncate a filename stem to fit within OS limits while preserving the extension.
///
/// If the stem exceeds the limit, it is truncated and `..` is appended to indicate
/// truncation. The extension is always preserved in full.
///
/// Returns the truncated stem.
pub fn truncate_stem(stem: &str, ext: &str) -> String {
    // Calculate available bytes for stem: MAX - ext bytes - dot - ".." suffix (2 bytes)
    let ext_bytes = ext.len();
    let dot_byte = if ext.is_empty() { 0 } else { 1 };
    let suffix_bytes = 2; // ".."
    let available = MAX_FILENAME_BYTES.saturating_sub(ext_bytes + dot_byte + suffix_bytes);

    if stem.len() <= available {
        return stem.to_string();
    }

    // Truncate at character boundary to avoid panicking on multi-byte UTF-8.
    // Walk back from the byte budget so the kept prefix never ends inside a
    // character: a char *starting* before the budget may end past it, which
    // would make the final filename exceed MAX_FILENAME_BYTES.
    let mut end = available;
    while end > 0 && !stem.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}..", &stem[..end])
}

/// Disambiguate a target path by adding `_1`, `_2`, etc. if it already exists.
///
/// If `target` doesn't exist, returns it unchanged. Otherwise, appends `_1`, `_2`,
/// etc. until a non-existing path is found.
///
/// Returns the disambiguated path.
pub fn disambiguate(target: &Path) -> PathBuf {
    if !target.exists() {
        return target.to_path_buf();
    }

    let parent = target.parent().unwrap_or(Path::new("."));
    let stem = target
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let ext = target
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    for i in 1.. {
        let candidate = parent.join(format!("{stem}_{i}{ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    // Fallback (shouldn't reach here in practice)
    target.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Assert that `stem` + optional dot + `ext` fits within the OS limit.
    fn assert_fits(stem: &str, ext: &str) {
        let full_len = if ext.is_empty() {
            stem.len()
        } else {
            stem.len() + 1 + ext.len()
        };
        assert!(
            full_len <= MAX_FILENAME_BYTES,
            "filename is {full_len} bytes (max {MAX_FILENAME_BYTES}): {stem:?}.{ext}"
        );
    }

    #[test]
    fn truncate_stem_short_name() {
        // Short names should be unchanged
        assert_eq!(truncate_stem("file", "txt"), "file");
        assert_eq!(truncate_stem("a", "md"), "a");
    }

    #[test]
    fn truncate_stem_at_limit() {
        // 251-byte stem with ".txt" is exactly 255 bytes; truncation keeps the
        // 249-byte budget and appends "..", still fitting the limit.
        let stem = "a".repeat(251);
        let result = truncate_stem(&stem, "txt");
        assert_eq!(result, format!("{}..", "a".repeat(249)));
        assert_fits(&result, "txt");
    }

    #[test]
    fn truncate_stem_exceeds_limit() {
        // Name exceeding limit should be truncated
        let stem = "a".repeat(300);
        let result = truncate_stem(&stem, "txt");
        assert_eq!(result, format!("{}..", "a".repeat(249)));
        assert_fits(&result, "txt");
    }

    #[test]
    fn truncate_stem_no_extension() {
        // No extension means more room for stem: 253 + ".." = 255 bytes
        let stem = "a".repeat(300);
        let result = truncate_stem(&stem, "");
        assert_eq!(result.len(), 255);
        assert!(result.ends_with(".."));
        assert_fits(&result, "");
    }

    #[test]
    fn truncate_stem_long_extension() {
        // Long extension reduces available stem space: 255 - 50 - 1 - 2 = 202
        let stem = "a".repeat(300);
        let ext = "x".repeat(50);
        let result = truncate_stem(&stem, &ext);
        assert_eq!(result.len(), 204);
        assert!(result.ends_with(".."));
        assert_fits(&result, &ext);
    }

    #[test]
    fn truncate_stem_exact_budget_unchanged() {
        // A stem of exactly `available` bytes needs no truncation
        let stem = "a".repeat(249);
        let result = truncate_stem(&stem, "txt");
        assert_eq!(result, stem);
        assert_fits(&result, "txt");
    }

    #[test]
    fn truncate_stem_multibyte_utf8() {
        // CJK characters are 3 bytes; the 249-byte budget is a char boundary,
        // so the name stays at 251 bytes (255 with the extension).
        let stem = "中".repeat(100); // 300 bytes
        let result = truncate_stem(&stem, "txt");
        assert_eq!(result.len(), 251);
        assert!(result.ends_with(".."));
        assert_fits(&result, "txt");
    }

    #[test]
    fn truncate_stem_emoji() {
        // Emoji are 4 bytes; the 249-byte budget lands inside the char at 248,
        // so the prefix is 248 bytes (62 emoji) and the name is 254 bytes.
        let stem = "😀".repeat(80); // 320 bytes
        let result = truncate_stem(&stem, "txt");
        assert_eq!(result.len(), 250);
        assert!(result.ends_with(".."));
        assert_fits(&result, "txt");
    }

    #[test]
    fn truncate_stem_emoji_crossing_budget() {
        // Regression: a 4-byte emoji starting one byte before the budget used
        // to be kept whole, producing a 258-byte filename.
        let ext = "txt";
        let available = MAX_FILENAME_BYTES - ext.len() - 1 - 2; // 249
        let stem = format!("{}😀{}", "a".repeat(available - 1), "b".repeat(20));
        let result = truncate_stem(&stem, ext);
        assert_eq!(result, format!("{}..", "a".repeat(available - 1)));
        assert_fits(&result, ext);
    }

    #[test]
    fn truncate_stem_cjk_crossing_budget() {
        // Same regression with a 3-byte character: old code produced 257 bytes.
        let ext = "txt";
        let available = MAX_FILENAME_BYTES - ext.len() - 1 - 2; // 249
        let stem = format!("{}中{}", "a".repeat(available - 1), "b".repeat(20));
        let result = truncate_stem(&stem, ext);
        assert_eq!(result, format!("{}..", "a".repeat(available - 1)));
        assert_fits(&result, ext);
    }

    #[test]
    fn disambiguate_no_conflict() {
        let tmp = PathBuf::from("/tmp/nomforge_test_disambiguate_no_conflict");
        let target = tmp.join("file.txt");
        let result = disambiguate(&target);
        assert_eq!(result, target);
    }

    #[test]
    fn disambiguate_with_conflict() {
        let tmp = PathBuf::from("/tmp/nomforge_test_disambiguate_conflict");
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("file.txt"), "content").unwrap();

        let target = tmp.join("file.txt");
        let result = disambiguate(&target);
        assert_eq!(result, tmp.join("file_1.txt"));

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn disambiguate_multiple_conflicts() {
        let tmp = PathBuf::from("/tmp/nomforge_test_disambiguate_multi");
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("file.txt"), "content").unwrap();
        fs::write(tmp.join("file_1.txt"), "content").unwrap();
        fs::write(tmp.join("file_2.txt"), "content").unwrap();

        let target = tmp.join("file.txt");
        let result = disambiguate(&target);
        assert_eq!(result, tmp.join("file_3.txt"));

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn disambiguate_no_extension() {
        let tmp = PathBuf::from("/tmp/nomforge_test_disambiguate_no_ext");
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("file"), "content").unwrap();

        let target = tmp.join("file");
        let result = disambiguate(&target);
        assert_eq!(result, tmp.join("file_1"));

        fs::remove_dir_all(&tmp).unwrap();
    }
}
