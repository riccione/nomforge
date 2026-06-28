use std::path::{Path, PathBuf};

use crate::error::{NomforgeError, Result};
use crate::rules::{FileMetadata, RenameContext, RenameRule};

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
}

impl RenameEngine {
    pub fn new(rules: Vec<RenameRule>) -> Self {
        Self { rules }
    }

    /// Generate a dry-run preview of renames without mutating the filesystem.
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
        for rule in &self.rules {
            ctx.stem = current_stem.clone();
            let new_stem = rule.apply(&ctx)?;
            current_stem = new_stem;
        }

        // Determine final extension
        let final_ext = self.apply_extension_rules(&extension, &ctx);
        let target = if final_ext.is_empty() {
            parent_dir.join(&current_stem)
        } else {
            parent_dir.join(format!("{}.{}", current_stem, final_ext))
        };

        Ok(RenamePlan {
            source: path.to_path_buf(),
            target,
        })
    }

    fn apply_extension_rules(&self, original_ext: &str, ctx: &RenameContext) -> String {
        let mut current_ext = original_ext.to_string();
        for rule in &self.rules {
            if let RenameRule::ChangeExtension { new_ext } = rule {
                current_ext = match new_ext {
                    None => current_ext,
                    Some(ext) if ext.is_empty() => String::new(),
                    Some(ext) => ext.clone(),
                };
                // Update ctx for subsequent rules
                let _ = ctx;
            }
        }
        current_ext
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
    use crate::rules::RenameRule;
    use std::fs;

    fn setup_test_dir(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("file1.txt"), "content1").unwrap();
        fs::write(dir.join("file2.txt"), "content2").unwrap();
        fs::write(dir.join("file3.txt"), "content3").unwrap();
    }

    fn cleanup_test_dir(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn plan_with_no_rules_keeps_original() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_no_rules");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();

        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].source, tmp.join("file1.txt"));
        assert_eq!(plans[0].target, tmp.join("file1.txt"));

        cleanup_test_dir(&tmp);
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
    fn plan_with_multiple_rules_chain() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_chain");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![
            RenameRule::Prefix("pre_".into()),
            RenameRule::Suffix("_suf".into()),
        ]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();

        assert_eq!(plans[0].target, tmp.join("pre_file1_suf.txt"));

        cleanup_test_dir(&tmp);
    }

    #[test]
    fn plan_with_case_transform() {
        let tmp = PathBuf::from("/tmp/nomforge_test_plan_case");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![RenameRule::CaseTransform(crate::rules::Case::Upper)]);
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
            position: crate::rules::SeqPosition::Prefix,
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
    fn apply_no_change_still_succeeds() {
        let tmp = PathBuf::from("/tmp/nomforge_test_apply_noop");
        setup_test_dir(&tmp);

        let engine = RenameEngine::new(vec![]);
        let files = vec![tmp.join("file1.txt")];
        let plans = engine.plan(&files).unwrap();
        let results = engine.apply(&plans).unwrap();

        assert!(results[0].success);
        assert!(tmp.join("file1.txt").exists());

        cleanup_test_dir(&tmp);
    }
}
