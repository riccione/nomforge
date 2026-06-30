mod common;

use nomforge_core::{RenameEngine, RenameRule, ScanOptions, scan_files};

// Test 1: plan with empty files list returns error
#[test]
fn error_no_files_returns_error() {
    let engine = RenameEngine::new(vec![RenameRule::Prefix("pre_".into())]);
    let result = engine.plan(&[]);
    assert!(result.is_err());
}

// Test 2: filename too long returns error
#[test]
fn error_filename_too_long() {
    let (tmp, _) = common::create_test_dir(&[("a.txt", "c")]);
    // Create a file with a long name (240 bytes stem)
    let long_name = "a".repeat(240);
    std::fs::write(tmp.path().join(format!("{long_name}.txt")), "c").unwrap();

    // Prefix that pushes it over 255 bytes
    let engine = RenameEngine::new(vec![RenameRule::Prefix(
        "very_long_prefix_that_makes_this_exceed_the_limit_".into(),
    )]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let result = engine.plan(&files);
    assert!(result.is_err());
    match result.unwrap_err() {
        nomforge_core::error::NomforgeError::FilenameTooLong { filename, length } => {
            assert!(length > 255);
            assert!(filename.len() > 255);
        }
        e => panic!("Expected FilenameTooLong, got: {e}"),
    }
}

// Test 3: scan with invalid regex returns error
#[test]
fn error_invalid_include_regex() {
    let (tmp, _) = common::create_test_dir(&[("file.txt", "c")]);
    let options = ScanOptions {
        include_pattern: Some(r"[invalid".into()),
        ..Default::default()
    };
    let result = scan_files(tmp.path(), &options);
    assert!(result.is_err());
}

// Test 4: scan with invalid exclude regex returns error
#[test]
fn error_invalid_exclude_regex() {
    let (tmp, _) = common::create_test_dir(&[("file.txt", "c")]);
    let options = ScanOptions {
        exclude_pattern: Some(r"(unclosed".into()),
        ..Default::default()
    };
    let result = scan_files(tmp.path(), &options);
    assert!(result.is_err());
}

// Test 5: apply with no-op (source == target) succeeds
#[test]
fn edge_noop_rename_succeeds() {
    let (tmp, _) = common::create_test_dir(&[("file.txt", "c")]);
    let engine = RenameEngine::new(vec![]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    assert_eq!(results[0].source, results[0].target);
    assert_eq!(common::file_names(tmp.path()), vec!["file.txt"]);
    assert_eq!(common::read_content(&tmp.path().join("file.txt")), "c");
}

// Test 6: single file rename works
#[test]
fn edge_single_file() {
    let (tmp, _) = common::create_test_dir(&[("only.txt", "content")]);
    let engine = RenameEngine::new(vec![RenameRule::Prefix("pre_".into())]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    assert_eq!(common::file_names(tmp.path()), vec!["pre_only.txt"]);
    assert_eq!(
        common::read_content(&tmp.path().join("pre_only.txt")),
        "content"
    );
}

// Test 7: filename at exactly 255 bytes is allowed
#[test]
fn edge_filename_at_255_bytes() {
    let (tmp, _) = common::create_test_dir(&[]);
    // "a" * 251 + ".txt" = 255 bytes
    let stem = "a".repeat(251);
    std::fs::write(tmp.path().join(format!("{stem}.txt")), "c").unwrap();

    let engine = RenameEngine::new(vec![]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();

    assert_eq!(plans.len(), 1);
    let target_name = plans[0].target.file_name().unwrap().to_str().unwrap();
    assert_eq!(target_name.len(), 255);
}

// Test 8: empty directory scan returns empty list
#[test]
fn edge_empty_dir_scan() {
    let tmp = tempfile::TempDir::new().unwrap();
    let files = scan_files(tmp.path(), &Default::default()).unwrap();
    assert!(files.is_empty());
}

// Test 9: rules chain correctly - find/replace + suffix
#[test]
fn edge_rules_chain_find_suffix() {
    let (tmp, _) = common::create_test_dir(&[("test_file.txt", "c")]);
    let engine = RenameEngine::new(vec![
        RenameRule::FindReplace {
            find: "_".into(),
            replace: "-".into(),
        },
        RenameRule::Suffix("_v2".into()),
    ]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["test-file_v2.txt"]);
}

// Test 10: case transform + extension change
#[test]
fn edge_case_and_extension() {
    let (tmp, _) = common::create_test_dir(&[("my_file.txt", "c")]);
    let engine = RenameEngine::new(vec![
        RenameRule::CaseTransform(nomforge_core::rules::Case::Upper),
        RenameRule::ChangeExtension {
            new_ext: Some("md".into()),
        },
    ]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["MY_FILE.md"]);
}

// Test 11: scan with extension filter that matches nothing
#[test]
fn edge_extension_filter_no_match() {
    let (tmp, _) = common::create_test_dir(&[("a.txt", "c"), ("b.txt", "c")]);
    let options = ScanOptions {
        extensions: Some(vec!["jpg".into()]),
        ..Default::default()
    };
    let files = scan_files(tmp.path(), &options).unwrap();
    assert!(files.is_empty());
}

// Test 12: plan with multiple rules applied in order
#[test]
fn edge_multiple_rules_order() {
    let (tmp, _) = common::create_test_dir(&[("file.txt", "c")]);
    let engine = RenameEngine::new(vec![
        RenameRule::Prefix("A_".into()),
        RenameRule::Prefix("B_".into()),
    ]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    // B_ applied after A_
    assert_eq!(names, vec!["B_A_file.txt"]);
}

// Test 13: suffix with counter
#[test]
fn edge_suffix_and_counter() {
    let (tmp, _) = common::create_test_dir(&[("a.txt", "c1"), ("b.txt", "c2")]);
    let engine = RenameEngine::new(vec![
        RenameRule::Suffix("_copy".into()),
        RenameRule::NumberSequence {
            start: 1,
            padding: 2,
            position: nomforge_core::rules::SeqPosition::Prefix,
        },
    ]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results.iter().all(|r| r.success));
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["01a_copy.txt", "02b_copy.txt"]);
}
