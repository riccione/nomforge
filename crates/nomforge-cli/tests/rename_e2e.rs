mod common;

use std::path::PathBuf;

fn external_undo_path(tmp: &tempfile::TempDir) -> PathBuf {
    tmp.path().parent().unwrap().join(format!(
        "cli_undo_{}.json",
        tmp.path().file_name().unwrap().to_string_lossy()
    ))
}

// Test 1: dry-run shows preview without changing files
#[test]
fn cli_rename_dry_run_shows_preview() {
    let tmp = common::create_test_dir(&[("file1.txt", "content"), ("file2.txt", "content")]);

    let (exit_code, stdout, stderr) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--prefix",
        "pre_",
    ]);

    assert_eq!(exit_code, 0, "CLI should succeed. stderr: {stderr}");
    assert!(stdout.contains("file1.txt") || stderr.contains("file1.txt"));
    assert_eq!(
        common::file_names(tmp.path()),
        vec!["file1.txt", "file2.txt"]
    );
}

// Test 2: --apply actually renames files
#[test]
fn cli_rename_apply_renames_files() {
    let tmp = common::create_test_dir(&[("file1.txt", "content1"), ("file2.txt", "content2")]);
    let undo_path = external_undo_path(&tmp);

    let (exit_code, stdout, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--prefix",
        "renamed_",
        "--apply",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);

    assert_eq!(exit_code, 0, "CLI should succeed. stdout: {stdout}");
    let names = common::file_names(tmp.path());
    assert!(names.contains(&"renamed_file1.txt".to_string()));
    assert!(names.contains(&"renamed_file2.txt".to_string()));
    assert_eq!(
        common::read_content(&tmp.path().join("renamed_file1.txt")),
        "content1"
    );
    let _ = std::fs::remove_file(&undo_path);
}

// Test 3: find/replace rule
#[test]
fn cli_rename_find_replace() {
    let tmp = common::create_test_dir(&[("photo_001.jpg", "c"), ("photo_002.jpg", "c")]);
    let undo_path = external_undo_path(&tmp);

    let (exit_code, _, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--find",
        "photo",
        "--replace",
        "image",
        "--apply",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);

    assert_eq!(exit_code, 0);
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["image_001.jpg", "image_002.jpg"]);
    let _ = std::fs::remove_file(&undo_path);
}

// Test 4: regex rule
#[test]
fn cli_rename_regex() {
    let tmp = common::create_test_dir(&[("2024-01-15_report.txt", "c")]);
    let undo_path = external_undo_path(&tmp);

    let (exit_code, _, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--regex",
        r"(\d{4})-(\d{2})-(\d{2})",
        "--replacement",
        "${3}_${2}_${1}",
        "--apply",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);

    assert_eq!(exit_code, 0);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["15_01_2024_report.txt"]);
    let _ = std::fs::remove_file(&undo_path);
}

// Test 5: suffix rule
#[test]
fn cli_rename_suffix() {
    let tmp = common::create_test_dir(&[("doc.txt", "c")]);
    let undo_path = external_undo_path(&tmp);

    let (exit_code, _, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--suffix",
        "_backup",
        "--apply",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);

    assert_eq!(exit_code, 0);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["doc_backup.txt"]);
    let _ = std::fs::remove_file(&undo_path);
}

// Test 6: case transform
#[test]
fn cli_rename_case_upper() {
    let tmp = common::create_test_dir(&[("hello.txt", "c")]);
    let undo_path = external_undo_path(&tmp);

    let (exit_code, _, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--case",
        "upper",
        "--apply",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);

    assert_eq!(exit_code, 0);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["HELLO.txt"]);
    let _ = std::fs::remove_file(&undo_path);
}

// Test 7: remove text
#[test]
fn cli_rename_remove() {
    let tmp = common::create_test_dir(&[("file_copy.txt", "c")]);
    let undo_path = external_undo_path(&tmp);

    let (exit_code, _, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--remove",
        "_copy",
        "--apply",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);

    assert_eq!(exit_code, 0);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["file.txt"]);
    let _ = std::fs::remove_file(&undo_path);
}

// Test 8: extension filter
#[test]
fn cli_rename_ext_filter() {
    let tmp = common::create_test_dir(&[("file1.txt", "c"), ("file2.jpg", "c")]);
    let undo_path = external_undo_path(&tmp);

    let (exit_code, _, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--prefix",
        "pre_",
        "--ext",
        "txt",
        "--apply",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);

    assert_eq!(exit_code, 0);
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["file2.jpg", "pre_file1.txt"]);
    let _ = std::fs::remove_file(&undo_path);
}

// Test 9: empty directory
#[test]
fn cli_rename_empty_dir() {
    let tmp = tempfile::TempDir::new().unwrap();

    let (exit_code, stdout, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--prefix",
        "pre_",
    ]);

    assert_eq!(exit_code, 0);
    assert!(
        stdout.contains("No files matched") || stdout.contains("0"),
        "stdout: {stdout}"
    );
}

// Test 10: nonexistent directory returns error
#[test]
fn cli_rename_nonexistent_dir() {
    let (exit_code, _, stderr) = common::run_nomforge(&[
        "rename",
        "--dir",
        "/tmp/nomforge_nonexistent_12345678",
        "--prefix",
        "pre_",
    ]);

    assert_ne!(exit_code, 0, "Should fail with nonexistent directory");
    assert!(
        stderr.contains("Walk directory error") || stderr.contains("not found"),
        "stderr: {stderr}"
    );
}

// Test 11: multiple rules chained (prefix + suffix)
#[test]
fn cli_rename_multiple_rules() {
    let tmp = common::create_test_dir(&[("file.txt", "c")]);
    let undo_path = external_undo_path(&tmp);

    let (exit_code, _, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--prefix",
        "pre_",
        "--suffix",
        "_suf",
        "--apply",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);

    assert_eq!(exit_code, 0);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["pre_file_suf.txt"]);
    let _ = std::fs::remove_file(&undo_path);
}

// Test 12: counter rule via CLI
#[test]
fn cli_rename_counter() {
    let tmp = common::create_test_dir(&[("a.txt", "c"), ("b.txt", "c")]);
    let undo_path = external_undo_path(&tmp);

    let (exit_code, _, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--counter-start",
        "1",
        "--counter-padding",
        "3",
        "--counter-position",
        "prefix",
        "--apply",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);

    assert_eq!(exit_code, 0);
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["001a.txt", "002b.txt"]);
    let _ = std::fs::remove_file(&undo_path);
}

// Test 13: no rules specified → error
#[test]
fn cli_rename_no_rules() {
    let tmp = common::create_test_dir(&[("file.txt", "c")]);

    let (exit_code, _, stderr) =
        common::run_nomforge(&["rename", "--dir", tmp.path().to_str().unwrap(), "--apply"]);

    assert_ne!(exit_code, 0, "Should fail with no rules");
    assert!(
        stderr.contains("No rename rules") || stderr.contains("rules"),
        "stderr: {stderr}"
    );
    // Files should be unchanged
    assert_eq!(common::file_names(tmp.path()), vec!["file.txt"]);
}

// Test 14: --verbose flag
#[test]
fn cli_rename_verbose() {
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
    assert!(
        stdout.contains("Undo history") || stdout.contains("undo"),
        "stdout: {stdout}"
    );
    let _ = std::fs::remove_file(&undo_path);
}

// Test 15: --no-undo flag
#[test]
fn cli_rename_no_undo() {
    let tmp = common::create_test_dir(&[("file.txt", "c")]);
    let undo_path = external_undo_path(&tmp);

    let (exit_code, _, _) = common::run_nomforge(&[
        "rename",
        "--dir",
        tmp.path().to_str().unwrap(),
        "--prefix",
        "pre_",
        "--apply",
        "--no-undo",
        "--history-file",
        undo_path.to_str().unwrap(),
    ]);

    assert_eq!(exit_code, 0);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["pre_file.txt"]);
    assert!(!undo_path.exists());
}
