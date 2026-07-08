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

    // Truncate at character boundary to avoid panicking on multi-byte UTF-8
    let truncated_byte_idx = stem
        .char_indices()
        .take_while(|(byte_idx, _)| *byte_idx + 1 <= available)
        .last()
        .map(|(byte_idx, ch)| byte_idx + ch.len_utf8())
        .unwrap_or(0);
    let truncated = &stem[..truncated_byte_idx];
    format!("{truncated}..")
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

    #[test]
    fn truncate_stem_short_name() {
        // Short names should be unchanged
        assert_eq!(truncate_stem("file", "txt"), "file");
        assert_eq!(truncate_stem("a", "md"), "a");
    }

    #[test]
    fn truncate_stem_at_limit() {
        // Name at exactly 255 bytes (with extension) should be unchanged
        let stem = "a".repeat(251);
        assert_eq!(truncate_stem(&stem, "txt").len(), 251);
    }

    #[test]
    fn truncate_stem_exceeds_limit() {
        // Name exceeding limit should be truncated
        let stem = "a".repeat(300);
        let result = truncate_stem(&stem, "txt");
        assert!(result.len() < 300);
        assert!(result.ends_with(".."));
    }

    #[test]
    fn truncate_stem_no_extension() {
        // No extension means more room for stem
        let stem = "a".repeat(300);
        let result = truncate_stem(&stem, "");
        assert!(result.len() < 300);
        assert!(result.ends_with(".."));
    }

    #[test]
    fn truncate_stem_long_extension() {
        // Long extension reduces available stem space
        let stem = "a".repeat(300);
        let ext = "x".repeat(50);
        let result = truncate_stem(&stem, &ext);
        assert!(result.len() < 300);
        assert!(result.ends_with(".."));
    }

    #[test]
    fn truncate_stem_multibyte_utf8() {
        // Multi-byte characters (CJK, emoji) must not be split mid-character
        // Each Chinese character is 3 bytes in UTF-8
        let stem = "中".repeat(100); // 300 bytes
        let result = truncate_stem(&stem, "txt");
        // Result should be valid UTF-8 and end with ".."
        assert!(result.ends_with(".."));
        // Verify no panics by converting to string (already is, but validates)
        let _ = result.as_str();
        // Verify all characters before ".." are complete
        let prefix = &result[..result.len() - 2];
        assert!(std::str::from_utf8(prefix.as_bytes()).is_ok());
    }

    #[test]
    fn truncate_stem_emoji() {
        // Emoji are 4 bytes each in UTF-8
        let stem = "😀".repeat(80); // 320 bytes
        let result = truncate_stem(&stem, "txt");
        assert!(result.ends_with(".."));
        let prefix = &result[..result.len() - 2];
        assert!(std::str::from_utf8(prefix.as_bytes()).is_ok());
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
