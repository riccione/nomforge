mod common;

use nomforge_core::{RenameEngine, RenameRule, conflict::ConflictReason, detect_conflicts};

// Test 1: no conflicts when all targets are unique
#[test]
fn conflict_no_conflicts_unique_targets() {
    let (tmp, _) =
        common::create_test_dir(&[("file1.txt", "a"), ("file2.txt", "b"), ("file3.txt", "c")]);
    let engine = RenameEngine::new(vec![RenameRule::Prefix("pre_".into())]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let conflicts = detect_conflicts(&plans);
    assert!(conflicts.is_empty());
}

// Test 2: SameTarget conflict when two files collide
#[test]
fn conflict_same_target_collision() {
    // Files "a.txt" and "A.txt" on case-insensitive FS would collide,
    // but we test the logical detection: two plans mapping to same target
    let (tmp, _) = common::create_test_dir(&[("file1.txt", "a"), ("file2.txt", "b")]);
    // Both files have same stem -> with suffix "_" they'd collide? No.
    // Let's manually create plans with same target.
    let plans = nomforge_core::engine::RenamePlan {
        source: tmp.path().join("file1.txt"),
        target: tmp.path().join("result.txt"),
    };
    let plans2 = nomforge_core::engine::RenamePlan {
        source: tmp.path().join("file2.txt"),
        target: tmp.path().join("result.txt"),
    };
    let conflicts = detect_conflicts(&[plans, plans2]);
    assert_eq!(conflicts.len(), 1);
    assert!(matches!(conflicts[0].reason, ConflictReason::SameTarget(_)));
}

// Test 3: TargetExists conflict when target already exists on disk
#[test]
fn conflict_target_exists_on_disk() {
    let (tmp, _) = common::create_test_dir(&[
        ("source.txt", "content"),
        ("existing.txt", "existing content"),
    ]);
    // Plan to rename source.txt -> existing.txt (which already exists)
    let plan = nomforge_core::engine::RenamePlan {
        source: tmp.path().join("source.txt"),
        target: tmp.path().join("existing.txt"),
    };
    let conflicts = detect_conflicts(&[plan]);
    assert_eq!(conflicts.len(), 1);
    assert!(matches!(
        conflicts[0].reason,
        ConflictReason::TargetExists(_)
    ));
}

// Test 4: no conflict when source == target (no-op)
#[test]
fn conflict_no_conflict_noop() {
    let (tmp, _) = common::create_test_dir(&[("file.txt", "c")]);
    let plan = nomforge_core::engine::RenamePlan {
        source: tmp.path().join("file.txt"),
        target: tmp.path().join("file.txt"),
    };
    let conflicts = detect_conflicts(&[plan]);
    assert!(conflicts.is_empty());
}

// Test 5: no conflicts with empty plans
#[test]
fn conflict_empty_plans() {
    let conflicts = detect_conflicts(&[]);
    assert!(conflicts.is_empty());
}

// Test 6: three plans with same target produce two conflicts
#[test]
fn conflict_triple_same_target() {
    let plans: Vec<_> = (0..3)
        .map(|i| nomforge_core::engine::RenamePlan {
            source: PathBuf::from(format!("/tmp/file{i}.txt")),
            target: PathBuf::from("/tmp/result.txt"),
        })
        .collect();
    let conflicts = detect_conflicts(&plans);
    assert_eq!(conflicts.len(), 2);
    for c in &conflicts {
        assert!(matches!(c.reason, ConflictReason::SameTarget(_)));
    }
}

// Test 7: mixed conflicts (same target + target exists)
#[test]
fn conflict_mixed_reasons() {
    let (tmp, _) =
        common::create_test_dir(&[("a.txt", "a"), ("b.txt", "b"), ("existing.txt", "exists")]);
    // Plan 1: a.txt -> result.txt
    // Plan 2: b.txt -> result.txt (SameTarget with plan 1)
    // Plan 3: (something) -> existing.txt (TargetExists)
    let plan1 = nomforge_core::engine::RenamePlan {
        source: tmp.path().join("a.txt"),
        target: tmp.path().join("result.txt"),
    };
    let plan2 = nomforge_core::engine::RenamePlan {
        source: tmp.path().join("b.txt"),
        target: tmp.path().join("result.txt"),
    };
    let plan3 = nomforge_core::engine::RenamePlan {
        source: tmp.path().join("a.txt"),
        target: tmp.path().join("existing.txt"),
    };
    let conflicts = detect_conflicts(&[plan1, plan2, plan3]);
    assert!(conflicts.len() >= 2);
    let has_same_target = conflicts
        .iter()
        .any(|c| matches!(c.reason, ConflictReason::SameTarget(_)));
    let has_target_exists = conflicts
        .iter()
        .any(|c| matches!(c.reason, ConflictReason::TargetExists(_)));
    assert!(has_same_target);
    assert!(has_target_exists);
}

// Test 8: conflict with real rename plans via engine
#[test]
fn conflict_detected_via_engine() {
    let (tmp, _) = common::create_test_dir(&[("alpha.txt", "a"), ("beta.txt", "b")]);
    let engine = RenameEngine::new(vec![RenameRule::FindReplace {
        find: "alpha".into(),
        replace: "beta".into(),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();

    // alpha.txt -> beta.txt, beta.txt stays beta.txt (no-op)
    assert_eq!(plans.len(), 2);
    let conflicts = detect_conflicts(&plans);
    assert!(!conflicts.is_empty());
}

// Test 9: TargetExists with engine-generated plans
#[test]
fn conflict_target_exists_via_engine() {
    let (tmp, _) = common::create_test_dir(&[("file1.txt", "a"), ("target.txt", "existing")]);
    // Rename file1.txt to target.txt (which already exists)
    let engine = RenameEngine::new(vec![RenameRule::FindReplace {
        find: "file1".into(),
        replace: "target".into(),
    }]);
    let files = nomforge_core::scan_files(tmp.path(), &Default::default()).unwrap();
    let plans = engine.plan(&files).unwrap();
    let conflicts = detect_conflicts(&plans);
    // file1.txt -> target.txt should conflict with existing target.txt
    assert!(!conflicts.is_empty());
}

use std::path::PathBuf;
