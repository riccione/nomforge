mod common;

use std::fs;

use nomforge_core::{
    RenameEngine, RenameResult, RenameRule, default_undo_log_path, log_renames, revert_last,
    undo_count,
};

fn undo_path_for(tmp: &tempfile::TempDir) -> std::path::PathBuf {
    tmp.path().parent().unwrap().join(format!(
        "nomforge_test_undo_{}.json",
        tmp.path().file_name().unwrap().to_string_lossy()
    ))
}

// Test 1: rename -> log -> undo -> verify files restored
#[test]
fn undo_single_batch() {
    let (tmp, _) = common::create_test_dir(&[("file1.txt", "hello"), ("file2.txt", "world")]);
    let undo_path = undo_path_for(&tmp);

    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let engine = RenameEngine::new(vec![RenameRule::Prefix("renamed_".into())]);
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    log_renames(&undo_path, &results).unwrap();
    assert_eq!(undo_count(&undo_path).unwrap(), 1);

    // Verify files are renamed
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["renamed_file1.txt", "renamed_file2.txt"]);

    // Undo
    let reverted = revert_last(&undo_path).unwrap();
    assert_eq!(reverted, 2);

    // Verify files are restored
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["file1.txt", "file2.txt"]);

    // Verify content is preserved
    assert_eq!(common::read_content(&tmp.path().join("file1.txt")), "hello");
    assert_eq!(common::read_content(&tmp.path().join("file2.txt")), "world");

    let _ = fs::remove_file(&undo_path);
}

// Test 2: multi-batch undo
#[test]
fn undo_multi_batch() {
    let (tmp, _) = common::create_test_dir(&[("a.txt", "content_a"), ("b.txt", "content_b")]);
    let undo_path = undo_path_for(&tmp);

    // First batch: prefix "batch1_"
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let engine = RenameEngine::new(vec![RenameRule::Prefix("batch1_".into())]);
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();
    log_renames(&undo_path, &results).unwrap();

    assert_eq!(undo_count(&undo_path).unwrap(), 1);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["batch1_a.txt", "batch1_b.txt"]);

    // Second batch: prefix "batch2_"
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let engine = RenameEngine::new(vec![RenameRule::Prefix("batch2_".into())]);
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();
    log_renames(&undo_path, &results).unwrap();

    assert_eq!(undo_count(&undo_path).unwrap(), 2);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["batch2_batch1_a.txt", "batch2_batch1_b.txt"]);

    // Undo last batch (batch2)
    let reverted = revert_last(&undo_path).unwrap();
    assert_eq!(reverted, 2);
    assert_eq!(undo_count(&undo_path).unwrap(), 1);

    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["batch1_a.txt", "batch1_b.txt"]);

    // Undo first batch (batch1)
    let reverted = revert_last(&undo_path).unwrap();
    assert_eq!(reverted, 2);
    assert_eq!(undo_count(&undo_path).unwrap(), 0);

    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["a.txt", "b.txt"]);
    assert_eq!(common::read_content(&tmp.path().join("a.txt")), "content_a");

    let _ = fs::remove_file(&undo_path);
}

// Test 3: undo when target file was deleted
#[test]
fn undo_when_target_deleted() {
    let (tmp, _) = common::create_test_dir(&[("file1.txt", "content"), ("file2.txt", "content2")]);
    let undo_path = undo_path_for(&tmp);

    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let engine = RenameEngine::new(vec![RenameRule::Prefix("renamed_".into())]);
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();
    log_renames(&undo_path, &results).unwrap();

    // Delete one of the renamed files
    fs::remove_file(tmp.path().join("renamed_file1.txt")).unwrap();

    // Undo should handle the missing file gracefully
    let reverted = revert_last(&undo_path).unwrap();
    // file2 should be restored, file1 cannot be restored (renamed file was deleted)
    assert!(reverted <= 2);

    // file2 should be back
    let names = common::file_names(tmp.path());
    assert!(names.contains(&"file2.txt".to_string()));

    let _ = fs::remove_file(&undo_path);
}

// Test 4: undo count decreases after revert
#[test]
fn undo_count_decreases_after_revert() {
    let (tmp, _) = common::create_test_dir(&[("file.txt", "c")]);
    let undo_path = undo_path_for(&tmp);

    // Log 3 batches
    for i in 0..3 {
        let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
        let engine = RenameEngine::new(vec![RenameRule::Prefix(format!("batch{i}_"))]);
        let plans = engine.plan(&files).unwrap();
        let results = engine.apply(&plans).unwrap();
        log_renames(&undo_path, &results).unwrap();
    }

    assert_eq!(undo_count(&undo_path).unwrap(), 3);

    revert_last(&undo_path).unwrap();
    assert_eq!(undo_count(&undo_path).unwrap(), 2);

    revert_last(&undo_path).unwrap();
    assert_eq!(undo_count(&undo_path).unwrap(), 1);

    revert_last(&undo_path).unwrap();
    assert_eq!(undo_count(&undo_path).unwrap(), 0);

    let _ = fs::remove_file(&undo_path);
}

// Test 5: undo with no history errors
#[test]
fn undo_no_history_errors() {
    let tmp = tempfile::TempDir::new().unwrap();
    let undo_path = tmp.path().join("nonexistent.json");

    let result = revert_last(&undo_path);
    assert!(result.is_err());
}

// Test 6: log empty results is noop
#[test]
fn undo_log_empty_results_is_noop() {
    let tmp = tempfile::TempDir::new().unwrap();
    let undo_path = tmp.path().join("undo.json");

    let results: Vec<RenameResult> = vec![];
    log_renames(&undo_path, &results).unwrap();

    assert_eq!(undo_count(&undo_path).unwrap(), 0);
    assert!(!undo_path.exists());
}

// Test 7: log only successful renames
#[test]
fn undo_log_only_successful() {
    let (tmp, _) = common::create_test_dir(&[("file.txt", "c")]);
    let undo_path = undo_path_for(&tmp);

    let results = vec![
        RenameResult {
            source: tmp.path().join("file.txt"),
            target: tmp.path().join("renamed.txt"),
            success: true,
            error: None,
        },
        RenameResult {
            source: tmp.path().join("nonexistent.txt"),
            target: tmp.path().join("nonexistent_renamed.txt"),
            success: false,
            error: Some("not found".into()),
        },
    ];

    log_renames(&undo_path, &results).unwrap();
    assert_eq!(undo_count(&undo_path).unwrap(), 1);

    let _ = fs::remove_file(&undo_path);
}

// Test 8: default undo log path works
#[test]
fn undo_default_path_works() {
    let path = default_undo_log_path();
    assert!(path.to_string_lossy().contains("nomforge"));
    assert!(path.to_string_lossy().ends_with(".json"));
}
