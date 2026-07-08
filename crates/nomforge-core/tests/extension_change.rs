mod common;

use nomforge_core::{RenameEngine, RenameRule};

// Test 1: change extension
#[test]
fn extension_change_txt_to_md() {
    let (tmp, _) = common::create_test_dir(&[("readme.txt", "content"), ("notes.txt", "notes")]);
    let engine = RenameEngine::new(vec![RenameRule::ChangeExtension {
        new_ext: Some("md".into()),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results.iter().all(|r| r.success));
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["notes.md", "readme.md"]);
    // Content preserved
    assert_eq!(
        common::read_content(&tmp.path().join("readme.md")),
        "content"
    );
}

// Test 2: remove extension
#[test]
fn extension_remove() {
    let (tmp, _) = common::create_test_dir(&[("file.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::ChangeExtension {
        new_ext: Some("".into()),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["file"]);
}

// Test 3: extension change chained with prefix
#[test]
fn extension_change_with_prefix() {
    let (tmp, _) = common::create_test_dir(&[("doc.txt", "c")]);
    let engine = RenameEngine::new(vec![
        RenameRule::Prefix("backup_".into()),
        RenameRule::ChangeExtension {
            new_ext: Some("bak".into()),
        },
    ]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["backup_doc.bak"]);
}

// Test 4: extension change with no extension file
#[test]
fn extension_no_existing_ext() {
    let (tmp, _) = common::create_test_dir(&[("Makefile", "all: build")]);
    let engine = RenameEngine::new(vec![RenameRule::ChangeExtension {
        new_ext: Some("mk".into()),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["Makefile.mk"]);
}

// Test 5: multiple extension changes (last one wins)
#[test]
fn extension_multiple_changes() {
    let (tmp, _) = common::create_test_dir(&[("file.txt", "c")]);
    let engine = RenameEngine::new(vec![
        RenameRule::ChangeExtension {
            new_ext: Some("md".into()),
        },
        RenameRule::ChangeExtension {
            new_ext: Some("html".into()),
        },
    ]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["file.html"]);
}

// Test 6: extension change preserves content
#[test]
fn extension_preserves_content() {
    let (tmp, _) = common::create_test_dir(&[("data.csv", "col1,col2\n1,2")]);
    let engine = RenameEngine::new(vec![RenameRule::ChangeExtension {
        new_ext: Some("tsv".into()),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    assert_eq!(
        common::read_content(&tmp.path().join("data.tsv")),
        "col1,col2\n1,2"
    );
}

// Test 7: extension change with regex
#[test]
fn extension_change_with_regex() {
    let (tmp, _) = common::create_test_dir(&[("photo_001.jpg", "c"), ("photo_002.jpg", "c")]);
    let engine = RenameEngine::new(vec![
        RenameRule::RegexReplace {
            pattern: r"photo_(\d+)".into(),
            replacement: "image_$1".into(),
        },
        RenameRule::ChangeExtension {
            new_ext: Some("png".into()),
        },
    ]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results.iter().all(|r| r.success));
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["image_001.png", "image_002.png"]);
}

// Test 8: many files extension change
#[test]
fn extension_many_files() {
    let (tmp, _) = common::create_test_dir(&[
        ("a.txt", "c1"),
        ("b.txt", "c2"),
        ("c.txt", "c3"),
        ("d.txt", "c4"),
    ]);
    let engine = RenameEngine::new(vec![RenameRule::ChangeExtension {
        new_ext: Some("md".into()),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert_eq!(results.len(), 4);
    assert!(results.iter().all(|r| r.success));
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["a.md", "b.md", "c.md", "d.md"]);
}

// Test 9: extension change with find/replace
#[test]
fn extension_change_with_find_replace() {
    let (tmp, _) = common::create_test_dir(&[("document_final.txt", "c")]);
    let engine = RenameEngine::new(vec![
        RenameRule::FindReplace {
            find: "_final".into(),
            replace: "".into(),
        },
        RenameRule::ChangeExtension {
            new_ext: Some("md".into()),
        },
    ]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["document.md"]);
}

// Test 10: extension same value is no-op
#[test]
fn extension_same_value_noop() {
    let (tmp, _) = common::create_test_dir(&[]);
    // Use a non-existing file to avoid disambiguation
    let engine = RenameEngine::new(vec![RenameRule::ChangeExtension {
        new_ext: Some("txt".into()),
    }]);
    let files = vec![tmp.path().join("file.txt")];
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    // Source should equal target (no change)
    assert_eq!(results[0].source, results[0].target);
}
