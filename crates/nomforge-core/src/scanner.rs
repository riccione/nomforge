use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

use crate::error::Result;

/// Options for scanning files in a directory.
#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    /// Scan subdirectories recursively.
    pub recursive: bool,
    /// Only include files matching this regex pattern.
    pub include_pattern: Option<String>,
    /// Exclude files matching this regex pattern.
    pub exclude_pattern: Option<String>,
    /// Only include files with these extensions (without dot).
    pub extensions: Option<Vec<String>>,
    /// Include hidden files (starting with `.`).
    pub include_hidden: bool,
}

/// Scan a directory for files matching the given options.
pub fn scan_files(dir: &Path, options: &ScanOptions) -> Result<Vec<PathBuf>> {
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

    let walker = if options.recursive {
        WalkDir::new(dir).follow_links(true)
    } else {
        WalkDir::new(dir).max_depth(1)
    };

    let mut files: Vec<PathBuf> = walker
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            options.include_hidden || !name.starts_with('.')
        })
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            match &include_re {
                Some(re) => re.is_match(&name),
                None => true,
            }
        })
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            match &exclude_re {
                Some(re) => !re.is_match(&name),
                None => true,
            }
        })
        .filter(|e| match &options.extensions {
            Some(exts) if !exts.is_empty() => e
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| exts.iter().any(|e| e.eq_ignore_ascii_case(ext)))
                .unwrap_or(false),
            _ => true,
        })
        .map(|e| e.into_path())
        .collect();

    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let tmp = PathBuf::from("/tmp/nomforge_test_scan_nonexistent_12345");
        let files = scan_files(&tmp, &Default::default()).unwrap();
        assert!(files.is_empty());
    }
}
