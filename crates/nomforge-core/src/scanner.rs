use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

use crate::error::Result;

/// Options for scanning files in a directory.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScanOptions {
    /// Scan subdirectories recursively.
    pub recursive: bool,
    /// Only include files matching this regex pattern.
    pub include_pattern: Option<String>,
    /// Exclude files matching this regex pattern.
    pub exclude_pattern: Option<String>,
    /// Only include files with these extensions (without dot).
    pub extensions: Option<Vec<String>>,
    /// Include hidden files and directories (names starting with `.`).
    ///
    /// When `false`, hidden directories are pruned entirely, so nothing inside
    /// them is scanned. The scan root itself is always exempt.
    pub include_hidden: bool,
}

/// Whether a file or directory name is hidden (starts with `.`).
fn is_hidden_name(name: &std::ffi::OsStr) -> bool {
    name.to_string_lossy().starts_with('.')
}

/// Scan a directory for files matching the given options.
///
/// Symbolic links are never followed: symlinked directories are not descended
/// into and symlinked files are ignored, so a scan can only return files that
/// actually live inside `dir`. This also means symlink loops cannot abort a
/// scan.
pub fn scan_files(dir: &Path, options: &ScanOptions) -> Result<Vec<PathBuf>> {
    Ok(scan_files_with_warnings(dir, options)?.0)
}

/// Scan a directory, reporting entries that could not be read.
///
/// Like [`scan_files`], but unreadable entries (e.g. a subdirectory without
/// permission) are skipped and returned as warnings instead of aborting the
/// whole scan. Only a failure on the scan root itself is an error.
pub fn scan_files_with_warnings(
    dir: &Path,
    options: &ScanOptions,
) -> Result<(Vec<PathBuf>, Vec<String>)> {
    let include_re = options
        .include_pattern
        .as_deref()
        .map(Regex::new)
        .transpose()
        .map_err(|e| crate::error::NomforgeError::InvalidRegex {
            pattern: options.include_pattern.clone().unwrap_or_default(),
            reason: e.to_string(),
        })?;

    let exclude_re = options
        .exclude_pattern
        .as_deref()
        .map(Regex::new)
        .transpose()
        .map_err(|e| crate::error::NomforgeError::InvalidRegex {
            pattern: options.exclude_pattern.clone().unwrap_or_default(),
            reason: e.to_string(),
        })?;

    // `follow_links` stays at walkdir's default (false). A root that is itself
    // a symlink is still traversed (walkdir's `follow_root_links` default).
    let walker = if options.recursive {
        WalkDir::new(dir)
    } else {
        WalkDir::new(dir).max_depth(1)
    };

    let mut files = Vec::new();
    let mut warnings = Vec::new();

    for entry in walker.into_iter().filter_entry(|e| {
        // Prune hidden directories (so nothing inside them is visited) and
        // skip hidden files. The scan root is exempt: choosing a hidden
        // directory explicitly should still scan its contents.
        options.include_hidden || e.depth() == 0 || !is_hidden_name(e.file_name())
    }) {
        let entry = match entry {
            Ok(entry) => entry,
            // A root failure (missing or unreadable) makes the scan unusable.
            Err(e) if e.depth() == 0 => return Err(e.into()),
            Err(e) => {
                warnings.push(e.to_string());
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let name = entry.file_name().to_string_lossy();
        if let Some(re) = &include_re
            && !re.is_match(&name)
        {
            continue;
        }
        if let Some(re) = &exclude_re
            && re.is_match(&name)
        {
            continue;
        }
        if let Some(exts) = &options.extensions
            && !exts.is_empty()
        {
            let matches = entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| exts.iter().any(|e| e.eq_ignore_ascii_case(ext)));
            if !matches {
                continue;
            }
        }

        files.push(entry.into_path());
    }

    files.sort();
    Ok((files, warnings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::NomforgeError;
    use std::fs;

    fn setup_test_dir(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("file1.txt"), "content").unwrap();
        fs::write(dir.join("file2.txt"), "content").unwrap();
        fs::write(dir.join("file3.jpg"), "content").unwrap();
        fs::write(dir.join("image.png"), "content").unwrap();
        fs::write(dir.join(".hidden"), "content").unwrap();

        // Subdirectory
        fs::create_dir_all(dir.join("subdir")).unwrap();
        fs::write(dir.join("subdir").join("nested.txt"), "content").unwrap();
        fs::write(dir.join("subdir").join("nested.jpg"), "content").unwrap();
    }

    fn cleanup_test_dir(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn scan_all_files_non_recursive() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_nonrec");
        setup_test_dir(&tmp);

        let options = ScanOptions {
            recursive: false,
            ..Default::default()
        };
        let files = scan_files(&tmp, &options).unwrap();

        // Should find top-level files only (not hidden by default)
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file1.txt"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file2.txt"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file3.jpg"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "image.png"));
        // Should not find nested files
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "nested.txt"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn scan_recursive() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_rec");
        setup_test_dir(&tmp);

        let options = ScanOptions {
            recursive: true,
            ..Default::default()
        };
        let files = scan_files(&tmp, &options).unwrap();

        assert!(files.iter().any(|f| f.file_name().unwrap() == "nested.txt"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "nested.jpg"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn scan_include_hidden() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_hidden");
        setup_test_dir(&tmp);

        let options = ScanOptions {
            include_hidden: true,
            ..Default::default()
        };
        let files = scan_files(&tmp, &options).unwrap();

        assert!(files.iter().any(|f| f.file_name().unwrap() == ".hidden"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn scan_exclude_hidden_by_default() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_no_hidden");
        setup_test_dir(&tmp);

        let options = ScanOptions::default();
        let files = scan_files(&tmp, &options).unwrap();

        assert!(!files.iter().any(|f| f.file_name().unwrap() == ".hidden"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn scan_filter_by_extension() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_ext");
        setup_test_dir(&tmp);

        let options = ScanOptions {
            extensions: Some(vec!["txt".into()]),
            ..Default::default()
        };
        let files = scan_files(&tmp, &options).unwrap();

        assert!(files.iter().any(|f| f.file_name().unwrap() == "file1.txt"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file2.txt"));
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "file3.jpg"));
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "image.png"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn scan_filter_by_multiple_extensions() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_multi_ext");
        setup_test_dir(&tmp);

        let options = ScanOptions {
            extensions: Some(vec!["txt".into(), "jpg".into()]),
            ..Default::default()
        };
        let files = scan_files(&tmp, &options).unwrap();

        assert!(files.iter().any(|f| f.file_name().unwrap() == "file1.txt"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file3.jpg"));
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "image.png"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn scan_include_pattern() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_include");
        setup_test_dir(&tmp);

        let options = ScanOptions {
            include_pattern: Some(r"^file\d\.txt$".into()),
            ..Default::default()
        };
        let files = scan_files(&tmp, &options).unwrap();

        assert!(files.iter().any(|f| f.file_name().unwrap() == "file1.txt"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file2.txt"));
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "file3.jpg"));
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "image.png"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn scan_exclude_pattern() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_exclude");
        setup_test_dir(&tmp);

        let options = ScanOptions {
            exclude_pattern: Some(r"\.jpg$".into()),
            ..Default::default()
        };
        let files = scan_files(&tmp, &options).unwrap();

        assert!(files.iter().any(|f| f.file_name().unwrap() == "file1.txt"));
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "file3.jpg"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "image.png"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn scan_sorted_results() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_sorted");
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("zebra.txt"), "content").unwrap();
        fs::write(tmp.join("apple.txt"), "content").unwrap();
        fs::write(tmp.join("mango.txt"), "content").unwrap();

        let files = scan_files(&tmp, &Default::default()).unwrap();

        assert_eq!(files[0].file_name().unwrap(), "apple.txt");
        assert_eq!(files[1].file_name().unwrap(), "mango.txt");
        assert_eq!(files[2].file_name().unwrap(), "zebra.txt");

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn scan_empty_directory() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_empty");
        fs::create_dir_all(&tmp).unwrap();

        let files = scan_files(&tmp, &Default::default()).unwrap();
        assert!(files.is_empty());

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn scan_nonexistent_directory() {
        use std::error::Error;

        let tmp = PathBuf::from("/tmp/nomforge_test_scan_nonexistent_12345");
        let err = scan_files(&tmp, &Default::default()).unwrap_err();

        // The walkdir error is carried as a source instead of being flattened.
        assert!(matches!(err, NomforgeError::WalkDir(_)), "err: {err}");
        assert!(err.source().is_some(), "missing source chain: {err}");
    }

    #[test]
    fn scan_excludes_files_in_hidden_directories() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_hidden_dirs");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join(".git").join("objects")).unwrap();
        fs::create_dir_all(tmp.join(".venv").join("lib")).unwrap();
        fs::create_dir_all(tmp.join("visible")).unwrap();
        fs::write(tmp.join(".git").join("config"), "c").unwrap();
        fs::write(tmp.join(".git").join("objects").join("abc"), "c").unwrap();
        fs::write(tmp.join(".venv").join("lib").join("site.py"), "c").unwrap();
        fs::write(tmp.join("visible").join("ok.txt"), "c").unwrap();

        let options = ScanOptions {
            recursive: true,
            ..Default::default()
        };
        let files = scan_files(&tmp, &options).unwrap();

        assert_eq!(files, vec![tmp.join("visible").join("ok.txt")]);

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn scan_include_hidden_includes_files_in_hidden_directories() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_hidden_dirs_included");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(tmp.join(".git").join("config"), "c").unwrap();

        let options = ScanOptions {
            recursive: true,
            include_hidden: true,
            ..Default::default()
        };
        let files = scan_files(&tmp, &options).unwrap();

        assert!(files.iter().any(|f| f.ends_with(".git/config")));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn scan_hidden_root_still_scanned() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_hidden_root/.hidden_root");
        let _ = fs::remove_dir_all(tmp.parent().unwrap());
        fs::create_dir_all(tmp.join("nested")).unwrap();
        fs::write(tmp.join("child.txt"), "c").unwrap();
        fs::write(tmp.join("nested").join("child2.txt"), "c").unwrap();

        let options = ScanOptions {
            recursive: true,
            ..Default::default()
        };
        let files = scan_files(&tmp, &options).unwrap();

        // Selecting a hidden directory explicitly must not exclude its contents.
        assert!(files.iter().any(|f| f.ends_with("child.txt")));
        assert!(files.iter().any(|f| f.ends_with("nested/child2.txt")));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn scan_deeply_nested_hidden_directory_excluded() {
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_deep_hidden");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("a").join(".b").join("c")).unwrap();
        fs::create_dir_all(tmp.join("a").join("c")).unwrap();
        fs::write(tmp.join("a").join(".b").join("c").join("d.txt"), "c").unwrap();
        fs::write(tmp.join("a").join("c").join("d.txt"), "c").unwrap();

        let options = ScanOptions {
            recursive: true,
            ..Default::default()
        };
        let files = scan_files(&tmp, &options).unwrap();

        assert_eq!(files, vec![tmp.join("a").join("c").join("d.txt")]);

        cleanup_test_dir(&tmp);
    }

    #[cfg(unix)]
    #[test]
    fn scan_does_not_follow_symlinked_directories() {
        use std::os::unix::fs::symlink;

        let base = PathBuf::from("/tmp/nomforge_test_scan_symlink_dir");
        let _ = fs::remove_dir_all(&base);
        let root = base.join("root");
        let outside = base.join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(root.join("file.txt"), "c").unwrap();
        fs::write(outside.join("secret.txt"), "c").unwrap();
        symlink(&outside, root.join("link")).unwrap();

        let options = ScanOptions {
            recursive: true,
            ..Default::default()
        };
        let files = scan_files(&root, &options).unwrap();

        assert_eq!(files, vec![root.join("file.txt")]);

        let _ = fs::remove_dir_all(&base);
    }

    #[cfg(unix)]
    #[test]
    fn scan_skips_symlinked_files() {
        use std::os::unix::fs::symlink;

        let base = PathBuf::from("/tmp/nomforge_test_scan_symlink_file");
        let _ = fs::remove_dir_all(&base);
        let root = base.join("root");
        let outside = base.join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(root.join("real.txt"), "c").unwrap();
        fs::write(outside.join("outside.txt"), "c").unwrap();
        symlink(outside.join("outside.txt"), root.join("alias.txt")).unwrap();

        let options = ScanOptions {
            recursive: true,
            ..Default::default()
        };
        let files = scan_files(&root, &options).unwrap();

        assert_eq!(files, vec![root.join("real.txt")]);

        let _ = fs::remove_dir_all(&base);
    }

    #[cfg(unix)]
    #[test]
    fn scan_symlink_loop_does_not_fail() {
        use std::os::unix::fs::symlink;

        let tmp = PathBuf::from("/tmp/nomforge_test_scan_symlink_loop");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("file.txt"), "c").unwrap();
        // Self-referential symlink: following it would loop forever.
        symlink(&tmp, tmp.join("loop")).unwrap();

        let options = ScanOptions {
            recursive: true,
            ..Default::default()
        };
        let files = scan_files(&tmp, &options).unwrap();

        assert_eq!(files, vec![tmp.join("file.txt")]);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[cfg(unix)]
    #[test]
    fn scan_follows_symlinked_root() {
        use std::os::unix::fs::symlink;

        let base = PathBuf::from("/tmp/nomforge_test_scan_symlink_root");
        let _ = fs::remove_dir_all(&base);
        let real = base.join("real");
        fs::create_dir_all(&real).unwrap();
        fs::write(real.join("child.txt"), "c").unwrap();
        let link_root = base.join("link");
        symlink(&real, &link_root).unwrap();

        let options = ScanOptions {
            recursive: true,
            ..Default::default()
        };
        let files = scan_files(&link_root, &options).unwrap();

        // The explicitly selected root may itself be a symlink.
        assert_eq!(files, vec![link_root.join("child.txt")]);

        let _ = fs::remove_dir_all(&base);
    }

    #[cfg(unix)]
    #[test]
    fn scan_skips_unreadable_subdirectory() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = PathBuf::from("/tmp/nomforge_test_scan_unreadable");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("locked")).unwrap();
        fs::write(tmp.join("readable.txt"), "c").unwrap();
        fs::write(tmp.join("locked").join("hidden.txt"), "c").unwrap();

        let locked = tmp.join("locked");
        let original = fs::metadata(&locked).unwrap().permissions();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

        // Running as root bypasses permission checks; such environments cannot
        // exercise the skip path.
        let blocked = fs::read_dir(&locked).is_err();

        let options = ScanOptions {
            recursive: true,
            ..Default::default()
        };
        let (files, warnings) = scan_files_with_warnings(&tmp, &options).unwrap();

        fs::set_permissions(&locked, original).unwrap();
        cleanup_test_dir(&tmp);

        assert_eq!(files, vec![tmp.join("readable.txt")]);
        if blocked {
            assert_eq!(warnings.len(), 1, "warnings: {warnings:?}");
            assert!(warnings[0].contains("locked"), "warnings: {warnings:?}");
        }
    }
}
