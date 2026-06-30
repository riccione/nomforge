mod common;

use nomforge_core::{RenameEngine, RenameRule, rules::SeqPosition};

// Test 1: basic counter prefix
#[test]
fn counter_prefix() {
    let (tmp, _) =
        common::create_test_dir(&[("alpha.txt", "a"), ("beta.txt", "b"), ("gamma.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::NumberSequence {
        start: 1,
        padding: 3,
        position: SeqPosition::Prefix,
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results.iter().all(|r| r.success));
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["001alpha.txt", "002beta.txt", "003gamma.txt"]);
}

// Test 2: counter suffix
#[test]
fn counter_suffix() {
    let (tmp, _) = common::create_test_dir(&[("a.txt", "c"), ("b.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::NumberSequence {
        start: 10,
        padding: 2,
        position: SeqPosition::Suffix,
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results.iter().all(|r| r.success));
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["a10.txt", "b11.txt"]);
}

// Test 3: counter prefix without separator
#[test]
fn counter_prefix_direct() {
    let (tmp, _) = common::create_test_dir(&[("file1.txt", "c"), ("file2.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::NumberSequence {
        start: 1,
        padding: 2,
        position: SeqPosition::Prefix,
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results.iter().all(|r| r.success));
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["01file1.txt", "02file2.txt"]);
}

// Test 4: counter with zero padding
#[test]
fn counter_zero_padding() {
    let (tmp, _) = common::create_test_dir(&[("a.txt", "c"), ("b.txt", "c"), ("c.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::NumberSequence {
        start: 1,
        padding: 1,
        position: SeqPosition::Prefix,
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results.iter().all(|r| r.success));
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["1a.txt", "2b.txt", "3c.txt"]);
}

// Test 5: counter starting from custom value
#[test]
fn counter_custom_start() {
    let (tmp, _) = common::create_test_dir(&[("x.txt", "c"), ("y.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::NumberSequence {
        start: 100,
        padding: 3,
        position: SeqPosition::Prefix,
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results.iter().all(|r| r.success));
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["100x.txt", "101y.txt"]);
}

// Test 6: counter chained with case transform
#[test]
fn counter_chained_with_case() {
    let (tmp, _) = common::create_test_dir(&[("item.txt", "c"), ("product.txt", "c")]);
    let engine = RenameEngine::new(vec![
        RenameRule::NumberSequence {
            start: 1,
            padding: 2,
            position: SeqPosition::Prefix,
        },
        RenameRule::CaseTransform(nomforge_core::rules::Case::Upper),
    ]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results.iter().all(|r| r.success));
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(names, vec!["01ITEM.txt", "02PRODUCT.txt"]);
}

// Test 7: counter with extension change
#[test]
fn counter_with_extension_change() {
    let (tmp, _) = common::create_test_dir(&[("file.txt", "c")]);
    let engine = RenameEngine::new(vec![
        RenameRule::NumberSequence {
            start: 1,
            padding: 3,
            position: SeqPosition::Prefix,
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
    assert_eq!(names, vec!["001file.md"]);
}

// Test 8: single file counter
#[test]
fn counter_single_file() {
    let (tmp, _) = common::create_test_dir(&[("only.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::NumberSequence {
        start: 42,
        padding: 3,
        position: SeqPosition::Prefix,
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    let names = common::file_names(tmp.path());
    assert_eq!(names, vec!["042only.txt"]);
}

// Test 9: counter preserves file content
#[test]
fn counter_preserves_content() {
    let (tmp, _) = common::create_test_dir(&[("doc.txt", "important content")]);
    let engine = RenameEngine::new(vec![RenameRule::NumberSequence {
        start: 1,
        padding: 2,
        position: SeqPosition::Prefix,
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert!(results[0].success);
    assert_eq!(
        common::read_content(&tmp.path().join("01doc.txt")),
        "important content"
    );
}

// Test 10: counter with many files
#[test]
fn counter_many_files() {
    let (tmp, _) = common::create_test_dir(&[
        ("a.txt", "c1"),
        ("b.txt", "c2"),
        ("c.txt", "c3"),
        ("d.txt", "c4"),
        ("e.txt", "c5"),
    ]);
    let engine = RenameEngine::new(vec![RenameRule::NumberSequence {
        start: 1,
        padding: 3,
        position: SeqPosition::Prefix,
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let results = engine.apply(&plans).unwrap();

    assert_eq!(results.len(), 5);
    assert!(results.iter().all(|r| r.success));
    let mut names = common::file_names(tmp.path());
    names.sort();
    assert_eq!(
        names,
        vec!["001a.txt", "002b.txt", "003c.txt", "004d.txt", "005e.txt"]
    );
    // Verify content preserved
    assert_eq!(common::read_content(&tmp.path().join("001a.txt")), "c1");
    assert_eq!(common::read_content(&tmp.path().join("005e.txt")), "c5");
}
