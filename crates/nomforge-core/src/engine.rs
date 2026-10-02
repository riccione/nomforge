use std::path::{Path, PathBuf};

use crate::display::{disambiguate, truncate_stem};
use crate::error::{NomforgeError, Result};
use crate::rules::{FileMetadata, RegexCache, RenameContext, RenameRule};

/// A planned rename operation for a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamePlan {
    pub source: PathBuf,
    pub target: PathBuf,
}

/// Result of applying a rename plan.
#[derive(Debug, Clone)]
pub struct RenameResult {
    pub source: PathBuf,
    pub target: PathBuf,
    pub success: bool,
    pub error: Option<String>,
}

/// The rename engine: generates plans and applies renames.
pub struct RenameEngine {
    rules: Vec<RenameRule>,
    cache: RegexCache,
}

impl RenameEngine {
    pub fn new(rules: Vec<RenameRule>) -> Self {
        let cache = RegexCache::from_rules(&rules);
        Self { rules, cache }
    }

    /// Generate a dry-run preview of renames without mutating the filesystem.
    ///
    /// Plans whose rules change nothing keep `source == target` and are never
    /// rewritten by disambiguation, even when the file exists on disk.
    pub fn plan(&self, files: &[PathBuf]) -> Result<Vec<RenamePlan>> {
        if files.is_empty() {
            return Err(NomforgeError::NoFilesFound);
        }

        let mut plans = Vec::with_capacity(files.len());

        for (counter, path) in files.iter().enumerate() {
            let plan = self.plan_single(path, counter)?;
            plans.push(plan);
        }

        Ok(plans)
    }

    /// Apply the given rename plans to the filesystem.
    ///
    /// Continues past per-plan failures; inspect `RenameResult::success`. A
    /// plan whose target already exists on disk (including one created
    /// earlier in the same batch) fails instead of overwriting. Only setup
    /// errors are returned as `Err`.
    pub fn apply(&self, plans: &[RenamePlan]) -> Result<Vec<RenameResult>> {
        let mut results = Vec::with_capacity(plans.len());

        for plan in plans {
            let result = self.apply_single(plan);
            results.push(result);
        }

        Ok(results)
    }

    fn plan_single(&self, path: &Path, counter: usize) -> Result<RenamePlan> {
        let filename = path
            .file_name()
            .ok_or_else(|| NomforgeError::FileNotFound(path.to_path_buf()))?
            .to_string_lossy()
            .into_owned();

        let stem = path
            .file_stem()
            .ok_or_else(|| NomforgeError::FileNotFound(path.to_path_buf()))?
            .to_string_lossy()
            .into_owned();

        let extension = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        let parent_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();

        let metadata = std::fs::metadata(path)
            .map(|m| FileMetadata {
                size: m.len(),
                modified: m.modified().ok(),
                created: m.created().ok(),
            })
            .unwrap_or(FileMetadata {
                size: 0,
                modified: None,
                created: None,
            });

        let mut ctx = RenameContext {
            filename,
            stem: stem.clone(),
            extension: extension.clone(),
            parent_dir: parent_dir.clone(),
            counter,
            metadata,
        };

        // Apply rules in order, each rule sees the result of the previous one
        let mut current_stem = stem;
        let mut current_ext = extension.clone();
        for rule in &self.rules {
            ctx.stem = current_stem.clone();
            ctx.extension = current_ext.clone();
            let new_stem = rule.apply(&ctx, &self.cache)?;
            current_stem = new_stem;
            // Track extension changes from ChangeExtension rules
            if let RenameRule::ChangeExtension { new_ext } = rule {
                current_ext = match new_ext {
                    None => current_ext,
                    Some(ext) if ext.is_empty() => String::new(),
                    Some(ext) => ext.clone(),
                };
            }
        }
        // Rule output becomes a path component: reject empty/dot stems and
        // path separators (mirrors validate_extension's threat model).
        if current_stem.is_empty()
            || current_stem == "."
            || current_stem == ".."
            || current_stem.contains(['/', '\\', '\0'])
        {
            return Err(NomforgeError::Conflict(format!(
                "rules produced invalid name: {current_stem:?}"
            )));
        }
        let target = if current_ext.is_empty() {
            parent_dir.join(&current_stem)
        } else {
            parent_dir.join(format!("{}.{}", current_stem, current_ext))
        };

        // Truncate filename if it exceeds OS limits
        let target = if let Some(filename_os) = target.file_name() {
            let filename_bytes = filename_os.len();
            if filename_bytes > 255 {
                let truncated_stem = truncate_stem(&current_stem, &current_ext);
                if current_ext.is_empty() {
                    parent_dir.join(&truncated_stem)
                } else {
                    parent_dir.join(format!("{}.{}", truncated_stem, current_ext))
                }
            } else {
                target
            }
        } else {
            target
        };

        // Don't disambiguate a no-op plan: its target IS our own source,
        // which exists on disk by definition.
        let target = if target.as_path() == path {
            target
        } else {
            disambiguate(&target)
        };

        Ok(RenamePlan {
            source: path.to_path_buf(),
            target,
        })
    }

    fn apply_single(&self, plan: &RenamePlan) -> RenameResult {
        if plan.source == plan.target {
            return RenameResult {
                source: plan.source.clone(),
                target: plan.target.clone(),
                success: true,
                error: None,
            };
        }

        // Refuse to overwrite an existing target (e.g. the duplicate of a
        // plan that just ran in this batch). Residual TOCTOU: the target
        // could appear between this check and rename(2). Sequential apply
        // also does not resolve ordering dependencies between plans
        // (A->B fails while B still exists) — chained renames need ordered
        // application, tracked as a follow-up alongside disambiguation
        // removal.
        if plan.target.exists() {
            return RenameResult {
                source: plan.source.clone(),
                target: plan.target.clone(),
                success: false,
                error: Some(NomforgeError::TargetAlreadyExists(plan.target.clone()).to_string()),
            };
        }

        match std::fs::rename(&plan.source, &plan.target) {
            Ok(()) => RenameResult {
                source: plan.source.clone(),
                target: plan.target.clone(),
                success: true,
                error: None,
            },
            Err(e) => RenameResult {
                source: plan.source.clone(),
                target: plan.target.clone(),
                success: false,
                error: Some(e.to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{Case, SeqPosition};
    use std::fs;

    fn setup_test_dir(path: &Path) {
        let _ = fs::create_dir_all(path);
    }

    fn cleanup_test_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn plan_with_prefix_rule() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_prefix");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![RenameRule::Prefix("pre_".into())]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();

        assert_eq!(plans[0].target, tmp.join("pre_file1.txt"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_with_suffix_rule() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_suffix");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![RenameRule::Suffix("_suf".into())]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();

        assert_eq!(plans[0].target, tmp.join("file1_suf.txt"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_with_find_replace() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_find_replace");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![RenameRule::FindReplace {
            find: "file".into(),
            replace: "doc".into(),
        }]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();

        assert_eq!(plans[0].target, tmp.join("doc1.txt"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_with_remove_text() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_remove");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![RenameRule::RemoveText("_copy".into())]);
        let files = vec![tmp.join("file_copy.txt")];
        let plans = engine.plan(&files).unwrap();

        assert_eq!(plans[0].target, tmp.join("file.txt"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_with_case_transform() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_case");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![RenameRule::CaseTransform(Case::Upper)]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();

        assert_eq!(plans[0].target, tmp.join("FILE1.txt"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_with_extension_change() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_ext");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![RenameRule::ChangeExtension {
            new_ext: Some("md".into()),
        }]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();

        assert_eq!(plans[0].target, tmp.join("file1.md"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_with_extension_removal() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_ext_rm");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![RenameRule::ChangeExtension {
            new_ext: Some("".into()),
        }]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();

        assert_eq!(plans[0].target, tmp.join("file1"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_multiple_files_counter() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_counter");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![RenameRule::NumberSequence {
            start: 1,
            padding: 3,
            position: SeqPosition::Prefix,
        }]);
        let files = vec![
            tmp.join("file1.txt"),
            tmp.join("file2.txt"),
            tmp.join("file3.txt"),
        ];
        let plans = engine.plan(&files).unwrap();

        assert_eq!(plans[0].target, tmp.join("001file1.txt"));
        assert_eq!(plans[1].target, tmp.join("002file2.txt"));
        assert_eq!(plans[2].target, tmp.join("003file3.txt"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_no_files_returns_error() {
        let engine = RenameEngine::new(vec![]);
        let result = engine.plan(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn apply_renames_files() {
        let tmp = PathBuf::from("/tmp/nomforge_test_apply");
        setup_test_dir(&tmp);
        fs::write(tmp.join("file1.txt"), "content").unwrap();

        let engine = RenameEngine::new(vec![RenameRule::Prefix("renamed_".into())]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();
        let results = engine.apply(&plans).unwrap();

        assert!(results[0].success);
        assert!(tmp.join("renamed_file1.txt").exists());
        assert!(!tmp.join("file1.txt").exists());

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_truncates_long_filename() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_long_name");
        setup_test_dir(&tmp);

        // Create a file with a long name
        let long_name = "a".repeat(240);
        fs::write(tmp.join(format!("{long_name}.txt")), "content").unwrap();

        // Prefix that pushes it over 255 bytes
        let engine = RenameEngine::new(vec![RenameRule::Prefix(
            "very_long_prefix_that_makes_this_exceed_the_limit_".into(),
        )]);
        let files = vec![tmp.join(format!("{long_name}.txt"))];
        let plans = engine.plan(&files).unwrap();

        // Should succeed with truncated filename
        assert_eq!(plans.len(), 1);
        let target_name = plans[0].target.file_name().unwrap().to_str().unwrap();
        assert!(target_name.len() <= 255);
        assert!(target_name.ends_with(".txt"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_prefix_with_extension_change() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_prefix_ext");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![
            RenameRule::Prefix("pre_".into()),
            RenameRule::ChangeExtension {
                new_ext: Some("md".into()),
            },
        ]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();

        assert_eq!(plans[0].target, tmp.join("pre_file1.md"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_case_with_extension_removal() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_case_ext_rm");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![
            RenameRule::CaseTransform(Case::Upper),
            RenameRule::ChangeExtension {
                new_ext: Some("".into()),
            },
        ]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();

        // No extension, so no dot in filename
        assert_eq!(plans[0].target, tmp.join("FILE1"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_multiple_extension_changes_uses_last() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_multi_ext");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![
            RenameRule::ChangeExtension {
                new_ext: Some("md".into()),
            },
            RenameRule::ChangeExtension {
                new_ext: Some("rs".into()),
            },
        ]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();

        // Last extension rule wins
        assert_eq!(plans[0].target, tmp.join("file1.rs"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_noop_rule_with_existing_file() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_noop_existing");
        setup_test_dir(&tmp);
        fs::write(tmp.join("file1.txt"), "content").unwrap();

        // Find doesn't match: the plan must stay a no-op even though the
        // source exists on disk (regression: disambiguate used to rewrite
        // it to file1_1.txt).
        let engine = RenameEngine::new(vec![RenameRule::FindReplace {
            find: "zzz".into(),
            replace: "yyy".into(),
        }]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();
        assert_eq!(plans[0].source, plans[0].target);

        let results = engine.apply(&plans).unwrap();
        assert!(results[0].success);
        assert!(tmp.join("file1.txt").exists());
        assert!(!tmp.join("file1_1.txt").exists());
        assert_eq!(
            fs::read_to_string(tmp.join("file1.txt")).unwrap(),
            "content"
        );

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_no_rules_with_existing_file() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_no_rules_existing");
        setup_test_dir(&tmp);
        fs::write(tmp.join("README"), "docs").unwrap();

        // No rules at all: the file must keep its name even though it
        // exists on disk.
        let engine = RenameEngine::new(vec![]);
        let files = vec![tmp.join("README")];
        let plans = engine.plan(&files).unwrap();
        assert_eq!(plans[0].source, plans[0].target);

        let results = engine.apply(&plans).unwrap();
        assert!(results[0].success);
        assert!(tmp.join("README").exists());
        assert!(!tmp.join("README_1").exists());

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn apply_fails_when_target_exists() {
        let tmp = PathBuf::from("/tmp/nomforge_test_apply_target_exists");
        setup_test_dir(&tmp);
        fs::write(tmp.join("source.txt"), "source content").unwrap();
        fs::write(tmp.join("existing.txt"), "existing content").unwrap();

        let engine = RenameEngine::new(vec![]);
        let plans = vec![RenamePlan {
            source: tmp.join("source.txt"),
            target: tmp.join("existing.txt"),
        }];
        let results = engine.apply(&plans).unwrap();

        assert!(!results[0].success);
        assert!(
            results[0]
                .error
                .as_deref()
                .is_some_and(|e| e.contains("already exists"))
        );
        // Neither file is touched
        assert_eq!(
            fs::read_to_string(tmp.join("source.txt")).unwrap(),
            "source content"
        );
        assert_eq!(
            fs::read_to_string(tmp.join("existing.txt")).unwrap(),
            "existing content"
        );

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn apply_duplicate_targets_first_wins() {
        let tmp = PathBuf::from("/tmp/nomforge_test_apply_duplicate_targets");
        setup_test_dir(&tmp);
        fs::write(tmp.join("a.txt"), "A").unwrap();
        fs::write(tmp.join("b.txt"), "B").unwrap();

        // Both plans claim the same new target: the second rename must fail
        // instead of destroying the first file's content.
        let engine = RenameEngine::new(vec![]);
        let plans = vec![
            RenamePlan {
                source: tmp.join("a.txt"),
                target: tmp.join("out.txt"),
            },
            RenamePlan {
                source: tmp.join("b.txt"),
                target: tmp.join("out.txt"),
            },
        ];
        let results = engine.apply(&plans).unwrap();

        assert!(results[0].success);
        assert!(!results[1].success);
        assert_eq!(fs::read_to_string(tmp.join("out.txt")).unwrap(), "A");
        assert_eq!(fs::read_to_string(tmp.join("b.txt")).unwrap(), "B");
        assert!(!tmp.join("a.txt").exists());

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_rejects_traversal_stem() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_traversal_stem");
        setup_test_dir(&tmp);
        fs::write(tmp.join("doc.txt"), "d").unwrap();

        // join() does not normalize "..", so a rule-produced stem with a
        // separator could move files outside the scanned directory.
        let engine = RenameEngine::new(vec![RenameRule::FindReplace {
            find: "doc".into(),
            replace: "../evil".into(),
        }]);
        let err = engine.plan(&[tmp.join("doc.txt")]).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("invalid name"), "msg: {msg}");
        assert!(msg.contains("../evil"), "msg: {msg}");

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_rejects_backslash_stem() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_backslash_stem");
        setup_test_dir(&tmp);
        fs::write(tmp.join("doc.txt"), "d").unwrap();

        let engine = RenameEngine::new(vec![RenameRule::FindReplace {
            find: "doc".into(),
            replace: "..\\evil".into(),
        }]);
        let err = engine.plan(&[tmp.join("doc.txt")]).unwrap_err();
        assert!(err.to_string().contains("invalid name"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_rejects_empty_stem_no_extension() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_empty_stem_no_ext");
        setup_test_dir(&tmp);
        fs::write(tmp.join("LICENSE"), "lic").unwrap();

        // parent_dir.join("") is the directory itself; disambiguate would
        // rename the file to a garbage sibling like <parentdir>_1.
        let engine = RenameEngine::new(vec![RenameRule::RemoveText("LICENSE".into())]);
        let err = engine.plan(&[tmp.join("LICENSE")]).unwrap_err();
        assert!(err.to_string().contains("invalid name"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_rejects_empty_stem_with_extension() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_empty_stem_ext");
        setup_test_dir(&tmp);
        fs::write(tmp.join("README.md"), "docs").unwrap();

        // Empty stem with an extension would silently produce a dotfile
        // ("README.md" -> ".md").
        let engine = RenameEngine::new(vec![RenameRule::RemoveText("README".into())]);
        let err = engine.plan(&[tmp.join("README.md")]).unwrap_err();
        assert!(err.to_string().contains("invalid name"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_rejects_dot_stems() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_dot_stems");
        setup_test_dir(&tmp);
        fs::write(tmp.join("doc.txt"), "d").unwrap();

        // "." and ".." as the whole stem would resolve to the parent
        // directory itself or its parent.
        for replacement in [".", ".."] {
            let engine = RenameEngine::new(vec![RenameRule::FindReplace {
                find: "doc".into(),
                replace: replacement.into(),
            }]);
            let err = engine.plan(&[tmp.join("doc.txt")]).unwrap_err();
            assert!(err.to_string().contains("invalid name"));
        }

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_allows_embedded_dotdots() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_embedded_dotdots");
        setup_test_dir(&tmp);
        fs::write(tmp.join("abc.txt"), "x").unwrap();

        // "a..c" contains ".." but no separator: a single path component
        // and safe. Only exact "." / ".." and separators are rejected.
        let engine = RenameEngine::new(vec![RenameRule::FindReplace {
            find: "b".into(),
            replace: "..".into(),
        }]);
        let plans = engine.plan(&[tmp.join("abc.txt")]).unwrap();
        assert_eq!(plans[0].target, tmp.join("a..c.txt"));

        cleanup_test_dir(&tmp);
    }
}
