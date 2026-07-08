mod common;

use nomforge_core::{RenameEngine, RenameRule};

// Test 1: basic regex replacement
#[test]
fn regex_basic_replacement() {
    let (tmp, _) = common::create_test_dir(&[
        ("photo_2024_01_15.jpg", "c1"),
        ("photo_2024_02_20.jpg", "c2"),
    ]);
    let engine = RenameEngine::new(vec![RenameRule::RegexReplace {
        pattern: r"(\d{4})_(\d{2})_(\d{2})".into(),
        replacement: "$1-$2-$3".into(),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results.iter().all(|r| r.success));
    let names = common::file_names(tmp.path());
    assert!(names.contains(&"photo_2024-01-15.jpg".to_string()));
    assert!(names.contains(&"photo_2024-02-20.jpg".to_string()));
}

// Test 2: regex with capture groups and back-references
#[test]
fn regex_capture_groups() {
    let (tmp, _) = common::create_test_dir(&[("file_abc_123.txt", "c"), ("file_def_456.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::RegexReplace {
        pattern: r"file_(\w+)_(\d+)".into(),
        replacement: "${2}_${1}".into(),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results.iter().all(|r| r.success));
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["123_abc.txt", "456_def.txt"]);
}

// Test 3: regex no match leaves file unchanged
#[test]
fn regex_no_match_unchanged() {
    let (tmp, _) = common::create_test_dir(&[]);
    // Use non-existing files to avoid disambiguation
    let engine = RenameEngine::new(vec![RenameRule::RegexReplace {
        pattern: r"^\d+_".into(),
        replacement: "".into(),
    }]);
    let files = vec![tmp.path().join("readme.md"), tmp.path().join("license.txt")];
    let plans = engine.plan(&files).unwrap();

    // No files match, so plans are no-ops (source == target)
    assert!(plans.iter().all(|p| p.source == p.target));
}

// Test 4: regex replaces all occurrences in filename
#[test]
fn regex_replace_all() {
    let (tmp, _) = common::create_test_dir(&[("aaa_bbb_aaa.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::RegexReplace {
        pattern: "aaa".into(),
        replacement: "xxx".into(),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["xxx_bbb_xxx.txt"]);
}

// Test 5: regex with case-insensitive flag
#[test]
fn regex_case_insensitive() {
    let (tmp, _) = common::create_test_dir(&[("MyDocument.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::RegexReplace {
        pattern: r"(?i)^my".into(),
        replacement: "the".into(),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["theDocument.txt"]);
}

// Test 6: regex invalid pattern errors
#[test]
fn regex_invalid_pattern_errors() {
    let (tmp, _) = common::create_test_dir(&[("file.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::RegexReplace {
        pattern: r"[invalid".into(),
        replacement: "x".into(),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let result = engine.plan(&files);
    assert!(result.is_err());
}

// Test 7: regex chained with other rules
#[test]
fn regex_chained_with_prefix() {
    let (tmp, _) = common::create_test_dir(&[("file_001.txt", "c")]);
    let engine = RenameEngine::new(vec![
        RenameRule::RegexReplace {
            pattern: r"_00(\d)".into(),
            replacement: "-$1".into(),
        },
        RenameRule::Prefix("doc_".into()),
    ]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["doc_file-1.txt"]);
}

// Test 8: regex with special characters in replacement
#[test]
fn regex_special_chars_in_replacement() {
    let (tmp, _) = common::create_test_dir(&[("file.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::RegexReplace {
        pattern: "file".into(),
        replacement: "file (copy)".into(),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["file (copy).txt"]);
}

// Test 9: regex empty match still produces valid plan
#[test]
fn regex_empty_match() {
    let (tmp, _) = common::create_test_dir(&[("file.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::RegexReplace {
        pattern: r"^".into(),
        replacement: "prefix_".into(),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["prefix_file.txt"]);
}

// Test 10: regex multiple files with same pattern
#[test]
fn regex_multiple_files() {
    let (tmp, _) = common::create_test_dir(&[
        ("IMG_001.jpg", "c"),
        ("IMG_002.jpg", "c"),
        ("IMG_003.jpg", "c"),
    ]);
    let engine = RenameEngine::new(vec![RenameRule::RegexReplace {
        pattern: r"IMG_(\d+)".into(),
        replacement: "photo_$1".into(),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results.iter().all(|r| r.success));
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(
        names,
        vec!["photo_001.jpg", "photo_002.jpg", "photo_003.jpg"]
    );
}
