use std::collections::HashSet;
use std::path::PathBuf;

use crate::engine::RenamePlan;

/// A conflict detected between two rename plans.
#[derive(Debug, Clone)]
pub struct Conflict {
    pub plan_a: usize,
    pub plan_b: usize,
    pub reason: ConflictReason,
}

/// Why a conflict was detected.
#[derive(Debug, Clone)]
pub enum ConflictReason {
    /// Two plans map to the same target path.
    SameTarget(PathBuf),
    /// The target path already exists on disk.
    TargetExists(PathBuf),
}

impl std::fmt::Display for ConflictReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SameTarget(path) => write!(f, "duplicate target: {}", path.display()),
            Self::TargetExists(path) => write!(f, "target already exists: {}", path.display()),
        }
    }
}

/// Detect conflicts between rename plans.
///
/// Checks for:
/// - Duplicate target paths (two files mapping to the same name)
/// - Target paths that already exist on disk
///
/// Note: `TargetExists` detection has an inherent TOCTOU race — a file could
/// be created between planning and conflict detection. This is acceptable for
/// CLI usage where operations are sequential and short-lived.
pub fn detect_conflicts(plans: &[RenamePlan]) -> Vec<Conflict> {
    let mut conflicts = Vec::new();
    let mut seen_targets: HashSet<&PathBuf> = HashSet::new();

    for (i, plan_a) in plans.iter().enumerate() {
        // Check if target already exists on disk (and is different from source)
        if plan_a.source != plan_a.target && plan_a.target.exists() {
            conflicts.push(Conflict {
                plan_a: i,
                plan_b: i,
                reason: ConflictReason::TargetExists(plan_a.target.clone()),
            });
        }

        // Check for duplicate targets
        if !seen_targets.insert(&plan_a.target) {
            // Find the other plan with the same target
            for (j, plan_b) in plans.iter().enumerate() {
                if j < i && plan_a.target == plan_b.target {
                    conflicts.push(Conflict {
                        plan_a: j,
                        plan_b: i,
                        reason: ConflictReason::SameTarget(plan_a.target.clone()),
                    });
                    break;
                }
            }
        }
    }

    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn cleanup_test_dir(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn conflict_same_target() {
        let plans = vec![
            RenamePlan {
                source: PathBuf::from("/tmp/a.txt"),
                target: PathBuf::from("/tmp/result.txt"),
            },
            RenamePlan {
                source: PathBuf::from("/tmp/b.txt"),
                target: PathBuf::from("/tmp/result.txt"),
            },
        ];

        let conflicts = detect_conflicts(&plans);
        assert_eq!(conflicts.len(), 1);
        assert!(matches!(conflicts[0].reason, ConflictReason::SameTarget(_)));
    }

    #[test]
    fn conflict_target_exists() {
        let tmp = PathBuf::from("/tmp/nomforge_test_conflict_exists");
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("existing.txt"), "content").unwrap();

        let plans = vec![RenamePlan {
            source: tmp.join("other.txt"),
            target: tmp.join("existing.txt"),
        }];

        let conflicts = detect_conflicts(&plans);
        assert_eq!(conflicts.len(), 1);
        assert!(matches!(
            conflicts[0].reason,
            ConflictReason::TargetExists(_)
        ));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn no_conflict_when_different_targets() {
        let plans = vec![
            RenamePlan {
                source: PathBuf::from("/tmp/a.txt"),
                target: PathBuf::from("/tmp/a_renamed.txt"),
            },
            RenamePlan {
                source: PathBuf::from("/tmp/b.txt"),
                target: PathBuf::from("/tmp/b_renamed.txt"),
            },
        ];

        let conflicts = detect_conflicts(&plans);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn no_conflict_when_source_equals_target() {
        let plans = vec![RenamePlan {
            source: PathBuf::from("/tmp/same.txt"),
            target: PathBuf::from("/tmp/same.txt"),
        }];

        let conflicts = detect_conflicts(&plans);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn conflict_display_same_target() {
        let reason = ConflictReason::SameTarget(PathBuf::from("/tmp/result.txt"));
        let msg = format!("{}", reason);
        assert!(msg.contains("duplicate target"));
        assert!(msg.contains("result.txt"));
    }

    #[test]
    fn conflict_display_target_exists() {
        let reason = ConflictReason::TargetExists(PathBuf::from("/tmp/existing.txt"));
        let msg = format!("{}", reason);
        assert!(msg.contains("already exists"));
        assert!(msg.contains("existing.txt"));
    }

    #[test]
    fn multiple_same_target_conflicts() {
        let plans = vec![
            RenamePlan {
                source: PathBuf::from("/tmp/a.txt"),
                target: PathBuf::from("/tmp/result.txt"),
            },
            RenamePlan {
                source: PathBuf::from("/tmp/b.txt"),
                target: PathBuf::from("/tmp/result.txt"),
            },
            RenamePlan {
                source: PathBuf::from("/tmp/c.txt"),
                target: PathBuf::from("/tmp/result.txt"),
            },
        ];

        let conflicts = detect_conflicts(&plans);
        // Should detect 2 conflicts: a->b and a->c (or b->c)
        assert_eq!(conflicts.len(), 2);
        for c in &conflicts {
            assert!(matches!(c.reason, ConflictReason::SameTarget(_)));
        }
    }

    #[test]
    fn no_conflicts_empty_plans() {
        let conflicts = detect_conflicts(&[]);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn no_conflicts_single_plan() {
        let plans = vec![RenamePlan {
            source: PathBuf::from("/tmp/a.txt"),
            target: PathBuf::from("/tmp/b.txt"),
        }];

        let conflicts = detect_conflicts(&plans);
        assert!(conflicts.is_empty());
    }
}
