mod common;

use nomforge_core::{ScanOptions, scan_files};

// Test 1: scan all files in flat directory
#[test]
fn scanner_flat_directory() {
    let (tmp, _) = common::create_test_dir(&[("a.txt", "c1"), ("b.txt", "c2"), ("c.jpg", "c3")]);
    let files = scan_files(tmp.path(), &Default::default()).unwrap();
    let names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    let mut names = names;
    names.sort();
    assert_eq!(names, vec!["a.txt", "b.txt", "c.jpg"]);
}

// Test 2: scan excludes hidden files by default
#[test]
fn scanner_excludes_hidden() {
    let (tmp, _) = common::create_test_dir(&[("visible.txt", "c")]);
    // Create hidden file manually
    std::fs::write(tmp.path().join(".hidden"), "secret").unwrap();

    let files = scan_files(tmp.path(), &Default::default()).unwrap();
    let names: Vec<_> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    assert_eq!(names, vec!["visible.txt"]);
    assert!(!names.contains(&".hidden".to_string()));
}

// Test 3: scan includes hidden when requested
#[test]
fn scanner_includes_hidden() {
    let (tmp, _) = common::create_test_dir(&[("visible.txt", "c")]);
    std::fs::write(tmp.path().join(".hidden"), "secret").unwrap();

    let options = ScanOptions {
        include_hidden: true,
        ..Default::default()
    };
    let files = scan_files(tmp.path(), &options).unwrap();
    let names: Vec<_> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    assert!(names.contains(&".hidden".to_string()));
    assert!(names.contains(&"visible.txt".to_string()));
}

// Test 4: scan with extension filter
#[test]
fn scanner_extension_filter() {
    let (tmp, _) = common::create_test_dir(&[
        ("a.txt", "c"),
        ("b.txt", "c"),
        ("c.jpg", "c"),
        ("d.png", "c"),
    ]);
    let options = ScanOptions {
        extensions: Some(vec!["txt".into()]),
        ..Default::default()
    };
    let files = scan_files(tmp.path(), &options).unwrap();
    let names: Vec<_> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    let mut names = names;
    names.sort();
    assert_eq!(names, vec!["a.txt", "b.txt"]);
}

// Test 5: scan with multiple extension filter
#[test]
fn scanner_multiple_extensions() {
    let (tmp, _) = common::create_test_dir(&[("a.txt", "c"), ("b.jpg", "c"), ("c.png", "c")]);
    let options = ScanOptions {
        extensions: Some(vec!["txt".into(), "png".into()]),
        ..Default::default()
    };
    let files = scan_files(tmp.path(), &options).unwrap();
    let names: Vec<_> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    let mut names = names;
    names.sort();
    assert_eq!(names, vec!["a.txt", "c.png"]);
}

// Test 6: scan with include regex pattern
#[test]
fn scanner_include_pattern() {
    let (tmp, _) = common::create_test_dir(&[
        ("photo_001.jpg", "c"),
        ("photo_002.jpg", "c"),
        ("doc.txt", "c"),
    ]);
    let options = ScanOptions {
        include_pattern: Some(r"^photo_".into()),
        ..Default::default()
    };
    let files = scan_files(tmp.path(), &options).unwrap();
    let names: Vec<_> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    let mut names = names;
    names.sort();
    assert_eq!(names, vec!["photo_001.jpg", "photo_002.jpg"]);
}

// Test 7: scan with exclude regex pattern
#[test]
fn scanner_exclude_pattern() {
    let (tmp, _) = common::create_test_dir(&[("a.txt", "c"), ("b.txt", "c"), ("c.jpg", "c")]);
    let options = ScanOptions {
        exclude_pattern: Some(r"\.jpg$".into()),
        ..Default::default()
    };
    let files = scan_files(tmp.path(), &options).unwrap();
    let names: Vec<_> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    let mut names = names;
    names.sort();
    assert_eq!(names, vec!["a.txt", "b.txt"]);
}

// Test 8: scan non-recursive skips subdirs
#[test]
fn scanner_non_recursive() {
    let (tmp, _) = common::create_test_dir(&[("top.txt", "c")]);
    std::fs::create_dir_all(tmp.path().join("subdir")).unwrap();
    std::fs::write(tmp.path().join("subdir").join("nested.txt"), "c").unwrap();

    let options = ScanOptions {
        recursive: false,
        ..Default::default()
    };
    let files = scan_files(tmp.path(), &options).unwrap();
    let names: Vec<_> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    assert_eq!(names, vec!["top.txt"]);
}

// Test 9: scan recursive includes subdirs
#[test]
fn scanner_recursive() {
    let (tmp, _) = common::create_test_dir(&[("top.txt", "c")]);
    std::fs::create_dir_all(tmp.path().join("subdir")).unwrap();
    std::fs::write(tmp.path().join("subdir").join("nested.txt"), "c").unwrap();

    let options = ScanOptions {
        recursive: true,
        ..Default::default()
    };
    let files = scan_files(tmp.path(), &options).unwrap();
    let names: Vec<_> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    let mut names = names;
    names.sort();
    assert_eq!(names, vec!["nested.txt", "top.txt"]);
}

// Test 10: scan empty directory returns empty
#[test]
fn scanner_empty_directory() {
    let tmp = tempfile::TempDir::new().unwrap();
    let files = scan_files(tmp.path(), &Default::default()).unwrap();
    assert!(files.is_empty());
}

// Test 11: scan nonexistent directory returns empty
#[test]
fn scanner_nonexistent_directory() {
    let tmp = tempfile::TempDir::new().unwrap();
    let nonexistent = tmp.path().join("does_not_exist");
    let files = scan_files(&nonexistent, &Default::default()).unwrap();
    assert!(files.is_empty());
}

// Test 12: scan results are sorted
#[test]
fn scanner_sorted() {
    let (tmp, _) =
        common::create_test_dir(&[("zebra.txt", "c"), ("apple.txt", "c"), ("mango.txt", "c")]);
    let files = scan_files(tmp.path(), &Default::default()).unwrap();
    let names: Vec<_> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    assert_eq!(names, vec!["apple.txt", "mango.txt", "zebra.txt"]);
}

// Test 13: scan with combined filters
#[test]
fn scanner_combined_filters() {
    let (tmp, _) = common::create_test_dir(&[
        ("photo_a.jpg", "c"),
        ("photo_b.jpg", "c"),
        ("doc_a.txt", "c"),
        ("doc_b.txt", "c"),
    ]);
    let options = ScanOptions {
        include_pattern: Some(r"^photo_".into()),
        extensions: Some(vec!["jpg".into()]),
        ..Default::default()
    };
    let files = scan_files(tmp.path(), &options).unwrap();
    let names: Vec<_> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    let mut names = names;
    names.sort();
    assert_eq!(names, vec!["photo_a.jpg", "photo_b.jpg"]);
}
