mod common;

use std::path::PathBuf;

fn external_undo_path(tmp: &tempfile::TempDir) -> PathBuf {
    tmp.path().parent().unwrap().join(format!(
        "cli_undo_{}.json",
        tmp.path().file_name().unwrap().to_string_lossy()
    ))
}

// Test 1: rename then undo restores files
#[test]
fn cli_undo_restores_files() {
    let tmp = common::create_test_dir(&[("file1.txt", "content1"), ("file2.txt", "content2")]);
    let undo_path = external_undo_path(&tmp);

    // Rename
    let (exit_code, _, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--prefix",
        "backup_",
        "--apply",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);
    assert_eq!(exit_code, 0);
    let names = common::file_names(tmp.path());
    assert!(names.contains(&"backup_file1.txt".to_string()));

    // Undo
    let (exit_code, _, _) =
        common::run_nomforge(&["undo", "--history-file", undo_path.to_str().unwrap()]);
    assert_eq!(exit_code, 0);

    // Verify restored
    let names = common::file_names(tmp.path());
    assert!(names.contains(&"file1.txt".to_string()));
    assert!(names.contains(&"file2.txt".to_string()));
    assert_eq!(
        common::read_content(&tmp.path().join("file1.txt")),
        "content1"
    );
    let _ = std::fs::remove_file(&undo_path);
}

// Test 2: undo with no history prints message and succeeds
#[test]
fn cli_undo_no_history() {
    let (exit_code, stdout, _) = common::run_nomforge(&[
        "undo",
        "--history-file",
        "/tmp/nomforge_nonexistent_undo_test.json",
    ]);

    assert_eq!(exit_code, 0, "Undo with no history succeeds gracefully");
    assert!(
        stdout.contains("No undo history") || stdout.contains("no undo"),
        "stdout: {stdout}"
    );
}

// Test 3: rename with --apply and --verbose shows undo info
#[test]
fn cli_rename_verbose_shows_undo() {
    let tmp = common::create_test_dir(&[("file.txt", "c")]);
    let undo_path = external_undo_path(&tmp);

    let (exit_code, stdout, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--prefix",
        "pre_",
        "--apply",
        "--verbose",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);

    assert_eq!(exit_code, 0);
    // Should mention undo history
    assert!(
        stdout.contains("undo") || stdout.contains("Undo"),
        "stdout: {stdout}"
    );
    let _ = std::fs::remove_file(&undo_path);
}

// Test 4: invalid case transform errors
#[test]
fn cli_rename_invalid_case() {
    let tmp = common::create_test_dir(&[("file.txt", "c")]);

    let (exit_code, _, stderr) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--case",
        "invalid_case",
        "--apply",
    ]);

    assert_ne!(exit_code, 0);
    assert!(
        stderr.contains("Invalid case") || stderr.contains("case"),
        "stderr: {stderr}"
    );
}

// Test 5: invalid counter position errors
#[test]
fn cli_rename_invalid_counter_position() {
    let tmp = common::create_test_dir(&[("file.txt", "c")]);

    let (exit_code, _, stderr) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--counter-padding",
        "3",
        "--counter-position",
        "invalid",
        "--apply",
    ]);

    assert_ne!(exit_code, 0);
    assert!(
        stderr.contains("Invalid counter") || stderr.contains("counter"),
        "stderr: {stderr}"
    );
}

// Test 6: undo after multiple renames reverts last batch only
#[test]
fn cli_undo_multi_batch() {
    let tmp = common::create_test_dir(&[("file.txt", "c")]);
    let undo_path = external_undo_path(&tmp);

    // First rename: prefix "a_"
    let (exit_code, _, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--prefix",
        "a_",
        "--apply",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);
    assert_eq!(exit_code, 0);
    assert!(common::file_names(tmp.path()).contains(&"a_file.txt".to_string()));

    // Second rename: prefix "b_" on a_file.txt
    let (exit_code, _, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--prefix",
        "b_",
        "--apply",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);
    assert_eq!(exit_code, 0);
    assert!(common::file_names(tmp.path()).contains(&"b_a_file.txt".to_string()));

    // Undo: should revert last rename (b_)
    let (exit_code, _, _) =
        common::run_nomforge(&["undo", "--history-file", undo_path.to_str().unwrap()]);
    assert_eq!(exit_code, 0);
    assert!(common::file_names(tmp.path()).contains(&"a_file.txt".to_string()));
    assert!(!common::file_names(tmp.path()).contains(&"b_a_file.txt".to_string()));

    let _ = std::fs::remove_file(&undo_path);
}
