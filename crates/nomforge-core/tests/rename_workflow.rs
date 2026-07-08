mod common;

use nomforge_core::{Case, RenameEngine, RenameRule, ScanOptions, scan_files};

// Test 1: scan -> plan -> apply -> verify files renamed
#[test]
fn full_rename_workflow_prefix() {
    let (tmp, _) = common::create_test_dir(&[
        ("photo1.jpg", "content1"),
        ("photo2.jpg", "content2"),
        ("photo3.jpg", "content3"),
    ]);

    let scan_opts = ScanOptions::default();
    let files = scan_files(tmp.path(), &scan_opts).unwrap();
    assert_eq!(files.len(), 3);

    let engine = RenameEngine::new(vec![RenameRule::Prefix("vacation_".into())]);
    let plans = engine.plan(&files).unwrap();
    assert_eq!(plans.len(), 3);

    let results = engine.apply(&plans).unwrap();
    assert!(results.iter().all(|r| r.success));

    let names = common::file_names(tmp.path());
    assert_eq!(
        names,
        vec![
            "vacation_photo1.jpg",
            "vacation_photo2.jpg",
            "vacation_photo3.jpg"
        ]
    );
}

// Test 2: scan -> plan -> apply -> verify content preserved
#[test]
fn full_rename_preserves_content() {
    let (tmp, _) = common::create_test_dir(&[("doc1.txt", "hello world"), ("doc2.txt", "foo bar")]);

    let files = scan_files(tmp.path(), &Default::default()).unwrap();
    let engine = RenameEngine::new(vec![RenameRule::Prefix("prefix_".into())]);
    let plans = engine.plan(&files).unwrap();
    engine.apply(&plans).unwrap();

    assert_eq!(
        common::read_content(&tmp.path().join("prefix_doc1.txt")),
        "hello world"
    );
    assert_eq!(
        common::read_content(&tmp.path().join("prefix_doc2.txt")),
        "foo bar"
    );
}

// Test 3: full pipeline with multiple rule types
#[test]
fn full_rename_multiple_rules() {
    let (tmp, _) = common::create_test_dir(&[("file1.txt", "c1"), ("file2.txt", "c2")]);

    let files = scan_files(tmp.path(), &Default::default()).unwrap();
    let engine = RenameEngine::new(vec![
        RenameRule::Prefix("pre_".into()),
        RenameRule::Suffix("_suf".into()),
        RenameRule::CaseTransform(Case::Upper),
    ]);
    let plans = engine.plan(&files).unwrap();
    engine.apply(&plans).unwrap();

    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["PRE_FILE1_SUF.txt", "PRE_FILE2_SUF.txt"]);
}

// Test 4: dry-run does not modify files
#[test]
fn full_rename_dry_run() {
    let (tmp, _) = common::create_test_dir(&[("a.txt", "content"), ("b.txt", "content")]);

    let files = scan_files(tmp.path(), &Default::default()).unwrap();
    let engine = RenameEngine::new(vec![RenameRule::Prefix("new_".into())]);
    let plans = engine.plan(&files).unwrap();

    // Don't apply — just plan
    assert_eq!(plans.len(), 2);

    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["a.txt", "b.txt"]);
}

// Test 5: scan empty directory then plan
#[test]
fn full_rename_empty_dir() {
    let (tmp, _) = common::create_test_dir(&[]);

    let files = scan_files(tmp.path(), &Default::default()).unwrap();
    assert!(files.is_empty());

    let engine = RenameEngine::new(vec![RenameRule::Prefix("pre_".into())]);
    let result = engine.plan(&files);
    assert!(result.is_err());
}

// Test 6: rename with no rules keeps original names
#[test]
fn full_rename_no_rules() {
    let (tmp, _) = common::create_test_dir(&[]);
    // Use non-existing files to avoid disambiguation
    let files = vec![tmp.path().join("file1.txt"), tmp.path().join("file2.txt")];

    let engine = RenameEngine::new(vec![]);
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results.iter().all(|r| r.success));
    assert!(results.iter().all(|r| r.source == r.target));
}

// Test 7: scan with extension filter then rename
#[test]
fn full_rename_with_extension_filter() {
    let (tmp, _) = common::create_test_dir(&[
        ("photo1.jpg", "c1"),
        ("photo2.png", "c2"),
        ("doc1.txt", "c3"),
    ]);

    let scan_opts = ScanOptions {
        extensions: Some(vec!["jpg".into(), "png".into()]),
        ..Default::default()
    };
    let files = scan_files(tmp.path(), &scan_opts).unwrap();
    assert_eq!(files.len(), 2);

    let engine = RenameEngine::new(vec![RenameRule::Prefix("img_".into())]);
    let plans = engine.plan(&files).unwrap();
    engine.apply(&plans).unwrap();

    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["doc1.txt", "img_photo1.jpg", "img_photo2.png"]);
}

// Test 8: scan with include pattern then rename
#[test]
fn full_rename_with_include_pattern() {
    let (tmp, _) = common::create_test_dir(&[
        ("IMG_001.jpg", "c1"),
        ("IMG_002.jpg", "c2"),
        ("DOC_001.txt", "c3"),
    ]);

    let scan_opts = ScanOptions {
        include_pattern: Some(r"^IMG_".into()),
        ..Default::default()
    };
    let files = scan_files(tmp.path(), &scan_opts).unwrap();
    assert_eq!(files.len(), 2);

    let engine = RenameEngine::new(vec![RenameRule::Prefix("photo_".into())]);
    let plans = engine.plan(&files).unwrap();
    engine.apply(&plans).unwrap();

    let names = common::file_names(tmp.path());
    assert_eq!(
        names,
        vec!["DOC_001.txt", "photo_IMG_001.jpg", "photo_IMG_002.jpg"]
    );
}

// Test 9: scan with exclude pattern then rename
#[test]
fn full_rename_with_exclude_pattern() {
    let (tmp, _) = common::create_test_dir(&[
        ("file1.txt", "c1"),
        ("file2.txt", "c2"),
        ("backup_file1.txt", "c3"),
    ]);

    let scan_opts = ScanOptions {
        exclude_pattern: Some(r"^backup_".into()),
        ..Default::default()
    };
    let files = scan_files(tmp.path(), &scan_opts).unwrap();
    assert_eq!(files.len(), 2);

    let engine = RenameEngine::new(vec![RenameRule::Suffix("_v2".into())]);
    let plans = engine.plan(&files).unwrap();
    engine.apply(&plans).unwrap();

    let names = common::file_names(tmp.path());
    assert_eq!(
        names,
        vec!["backup_file1.txt", "file1_v2.txt", "file2_v2.txt"]
    );
}

// Test 10: recursive scan and rename in subdirectories
#[test]
fn full_rename_recursive() {
    let tmp = common::create_test_dir_nested(&[
        ("root.txt", "rc1"),
        ("sub/nested.txt", "rc2"),
        ("sub/deep/deep.txt", "rc3"),
    ]);

    let scan_opts = ScanOptions {
        recursive: true,
        ..Default::default()
    };
    let files = scan_files(tmp.path(), &scan_opts).unwrap();
    assert_eq!(files.len(), 3);

    let engine = RenameEngine::new(vec![RenameRule::Prefix("r_".into())]);
    let plans = engine.plan(&files).unwrap();
    engine.apply(&plans).unwrap();

    let names = common::file_names_recursive(tmp.path());
    assert!(names.contains(&"r_root.txt".to_string()));
    assert!(names.contains(&"sub/r_nested.txt".to_string()));
    assert!(names.contains(&"sub/deep/r_deep.txt".to_string()));
}

// Test 11: remove text rule on real files
#[test]
fn full_rename_remove_text() {
    let (tmp, _) = common::create_test_dir(&[("photo_copy.jpg", "c1"), ("doc_copy.txt", "c2")]);

    let files = scan_files(tmp.path(), &Default::default()).unwrap();
    let engine = RenameEngine::new(vec![RenameRule::RemoveText("_copy".into())]);
    let plans = engine.plan(&files).unwrap();
    engine.apply(&plans).unwrap();

    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["doc.txt", "photo.jpg"]);
}

// Test 12: find and replace on real files
#[test]
fn full_rename_find_replace() {
    let (tmp, _) = common::create_test_dir(&[("2024_photo.jpg", "c1"), ("2024_doc.txt", "c2")]);

    let files = scan_files(tmp.path(), &Default::default()).unwrap();
    let engine = RenameEngine::new(vec![RenameRule::FindReplace {
        find: "2024".into(),
        replace: "2025".into(),
    }]);
    let plans = engine.plan(&files).unwrap();
    engine.apply(&plans).unwrap();

    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["2025_doc.txt", "2025_photo.jpg"]);
}
