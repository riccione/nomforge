use eframe::egui;

use crate::state::State;

/// The nomforge GUI application.
#[derive(Default)]
pub struct NomforgeApp {
    state: State,
}

impl eframe::App for NomforgeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading(format!("nomforge v{}", { env!("CARGO_PKG_VERSION") }));
        ui.separator();

        // Directory picker
        crate::panels::folder_picker::show(ui, &mut self.state);

        ui.separator();

        // Rules section
        crate::panels::rule_builder::show(ui, &mut self.state);

        // Filtering section
        ui.collapsing("Filtering", |ui| {
            ui.horizontal(|ui| {
                ui.label("Filter by extension:");
                ui.text_edit_singleline(&mut self.state.filter_ext);
            });
            ui.horizontal(|ui| {
                ui.label("Include pattern:");
                ui.text_edit_singleline(&mut self.state.include);
            });
            ui.horizontal(|ui| {
                ui.label("Exclude pattern:");
                ui.text_edit_singleline(&mut self.state.exclude);
            });
            ui.checkbox(&mut self.state.recursive, "Recursive");
            ui.checkbox(&mut self.state.hidden, "Include hidden files");
        });

        // Undo settings
        ui.collapsing("Undo", |ui| {
            ui.checkbox(&mut self.state.no_undo, "Disable undo logging");
            ui.horizontal(|ui| {
                ui.label("History file:");
                ui.text_edit_singleline(&mut self.state.history_file);
            });
            if self.state.history_file.is_empty() {
                ui.label("(default: ~/.local/share/nomforge/undo_log.json)");
            }
            ui.separator();
            if ui.button("Undo Last Batch").clicked() {
                self.state.show_undo_modal = true;
            }
        });

        // Undo confirmation modal
        if self.state.show_undo_modal {
            let modal = egui::Modal::new(egui::Id::new("undo_confirm")).show(ui.ctx(), |ui| {
                ui.set_width(300.0);
                ui.heading("Undo Last Batch?");
                ui.label("This will revert the most recent rename operation.");
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        self.perform_undo();
                        self.state.show_undo_modal = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.state.show_undo_modal = false;
                    }
                });
            });
            if modal.should_close() {
                self.state.show_undo_modal = false;
            }
        }

        // Conflict confirmation modal
        self.show_conflict_modal(ui);

        ui.separator();

        // Actions
        ui.horizontal(|ui| {
            if ui.button("Scan").clicked() {
                self.scan_files();
            }
            if ui.button("Preview").clicked() {
                self.preview();
            }
            if ui.button("Apply").clicked() {
                self.apply();
            }
        });

        // Status bar
        crate::panels::status_bar::show(ui, &self.state);

        // Preview and results table
        crate::panels::preview_table::show(ui, &self.state);
    }
}

impl NomforgeApp {
    fn scan_files(&mut self) {
        self.state.reset_output();

        let dir = std::path::Path::new(&self.state.dir);
        if !dir.exists() {
            self.state.status = format!("Directory not found: {}", self.state.dir);
            return;
        }

        let scan_options = self.state.build_scan_options();
        match nomforge_core::scan_files(dir, &scan_options) {
            Ok(files) => {
                self.state.status = format!("Found {} file(s)", files.len());
                self.state.files = files;
            }
            Err(e) => {
                self.state.status = format!("Scan error: {e}");
            }
        }
    }

    fn preview(&mut self) {
        // Save cached files before reset
        let cached_files = if !self.state.files.is_empty() {
            Some(self.state.files.clone())
        } else {
            None
        };

        self.state.reset_output();

        let dir = std::path::Path::new(&self.state.dir);
        if !dir.exists() {
            self.state.status = format!("Directory not found: {}", self.state.dir);
            return;
        }

        let rules = match self.state.build_rules() {
            Ok(r) => r,
            Err(e) => {
                self.state.status = format!("Rule error: {e}");
                return;
            }
        };

        // Reuse cached files if available, otherwise scan
        let files = if let Some(cached) = cached_files {
            cached
        } else {
            let scan_options = self.state.build_scan_options();
            match nomforge_core::scan_files(dir, &scan_options) {
                Ok(f) => f,
                Err(e) => {
                    self.state.status = format!("Scan error: {e}");
                    return;
                }
            }
        };

        if files.is_empty() {
            self.state.status = "No files found".into();
            return;
        }

        let engine = nomforge_core::RenameEngine::new(rules);
        let plans = match engine.plan(&files) {
            Ok(p) => p,
            Err(e) => {
                self.state.status = format!("Plan error: {e}");
                return;
            }
        };

        self.state.status = format!("{} file(s) to rename", plans.len());

        // Detect conflicts for preview display
        let conflicts = nomforge_core::detect_conflicts(&plans);
        self.state.pending_conflicts = conflicts;
        self.state.plans = plans;
    }

    fn apply(&mut self) {
        // Save cached files before reset
        let cached_files = if !self.state.files.is_empty() {
            Some(self.state.files.clone())
        } else {
            None
        };

        self.state.reset_output();

        let dir = std::path::Path::new(&self.state.dir);
        if !dir.exists() {
            self.state.status = format!("Directory not found: {}", self.state.dir);
            return;
        }

        let rules = match self.state.build_rules() {
            Ok(r) => r,
            Err(e) => {
                self.state.status = format!("Rule error: {e}");
                return;
            }
        };

        // Reuse cached files if available, otherwise scan
        let files = if let Some(cached) = cached_files {
            cached
        } else {
            let scan_options = self.state.build_scan_options();
            match nomforge_core::scan_files(dir, &scan_options) {
                Ok(f) => f,
                Err(e) => {
                    self.state.status = format!("Scan error: {e}");
                    return;
                }
            }
        };

        if files.is_empty() {
            self.state.status = "No files found".into();
            return;
        }

        let engine = nomforge_core::RenameEngine::new(rules);
        let plans = match engine.plan(&files) {
            Ok(p) => p,
            Err(e) => {
                self.state.status = format!("Plan error: {e}");
                return;
            }
        };

        // Detect conflicts before applying
        let conflicts = nomforge_core::detect_conflicts(&plans);
        if !conflicts.is_empty() {
            self.state.pending_conflicts = conflicts;
            self.state.show_conflict_modal = true;
            self.state.status = format!(
                "{} conflict(s) detected — review before applying",
                self.state.pending_conflicts.len()
            );
            // Store plans and files for later apply
            self.state.plans = plans;
            self.state.files = files;
            return;
        }

        self.do_apply(plans, files);
    }

    fn perform_undo(&mut self) {
        let history_path = if self.state.history_file.is_empty() {
            nomforge_core::default_undo_log_path()
        } else {
            std::path::PathBuf::from(&self.state.history_file)
        };

        if !history_path.exists() {
            self.state.status = "No undo history found".into();
            return;
        }

        match nomforge_core::revert_last(&history_path) {
            Ok(0) => self.state.status = "No undo history found".into(),
            Ok(n) => {
                self.state.status = format!("Reverted {n} file(s)");
                self.preview();
            }
            Err(e) => self.state.status = format!("Undo error: {e}"),
        }
    }

    fn do_apply(&mut self, plans: Vec<nomforge_core::RenamePlan>, files: Vec<std::path::PathBuf>) {
        let engine = nomforge_core::RenameEngine::new(self.state.build_rules().unwrap_or_default());

        let results = match engine.apply(&plans) {
            Ok(r) => r,
            Err(e) => {
                self.state.status = format!("Apply error: {e}");
                return;
            }
        };

        let succeeded = results.iter().filter(|r| r.success).count();
        self.state.status = format!("Renamed {succeeded} file(s)");
        self.state.plans = plans;
        self.state.files = files;
        self.state.results = results.clone();
        self.state.applied = true;

        if !self.state.no_undo {
            let history_path = if self.state.history_file.is_empty() {
                nomforge_core::default_undo_log_path()
            } else {
                std::path::PathBuf::from(&self.state.history_file)
            };
            if let Err(e) = nomforge_core::log_renames(&history_path, &results) {
                self.state.status = format!("Renamed {succeeded} file(s) (undo log error: {e})");
            }
        }
    }

    fn show_conflict_modal(&mut self, ui: &mut egui::Ui) {
        if !self.state.show_conflict_modal {
            return;
        }

        let modal = egui::Modal::new(egui::Id::new("conflict_confirm")).show(ui.ctx(), |ui| {
            ui.set_width(400.0);
            ui.heading("Conflicts Detected");
            ui.label("The following conflicts were found:");
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for conflict in &self.state.pending_conflicts {
                        ui.label(format!("• {}", conflict.reason));
                    }
                });

            ui.add_space(16.0);
            ui.label("Apply anyway? Files with conflicts may be overwritten.");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Apply Anyway").clicked() {
                    let plans = std::mem::take(&mut self.state.plans);
                    let files = std::mem::take(&mut self.state.files);
                    self.state.show_conflict_modal = false;
                    self.state.pending_conflicts.clear();
                    self.do_apply(plans, files);
                }
                if ui.button("Cancel").clicked() {
                    self.state.show_conflict_modal = false;
                    self.state.pending_conflicts.clear();
                    self.state.status = "Apply cancelled".into();
                }
            });
        });

        if modal.should_close() {
            self.state.show_conflict_modal = false;
            self.state.pending_conflicts.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn app_default_state() {
        let app = NomforgeApp::default();
        assert!(app.state.dir.is_empty());
        assert_eq!(app.state.status, "Ready");
    }

    #[test]
    fn do_apply_logs_renames_and_allows_undo() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();

        // Create test files
        fs::write(dir.join("file1.txt"), "content1").unwrap();
        fs::write(dir.join("file2.txt"), "content2").unwrap();

        // Set up undo log path
        let undo_path = dir.join("undo_log.json");

        let mut app = NomforgeApp {
            state: State {
                dir: dir.to_string_lossy().into_owned(),
                prefix: "renamed_".into(),
                history_file: undo_path.to_string_lossy().into_owned(),
                ..Default::default()
            },
        };

        // Scan files
        app.scan_files();
        assert_eq!(app.state.files.len(), 2);

        // Build rules and plans
        let rules = app.state.build_rules().unwrap();
        let engine = nomforge_core::RenameEngine::new(rules);
        let plans = engine.plan(&app.state.files).unwrap();
        assert_eq!(plans.len(), 2);

        let files = app.state.files.clone();

        // Apply renames with undo logging
        app.do_apply(plans, files);

        // Verify renames succeeded
        assert!(dir.join("renamed_file1.txt").exists());
        assert!(dir.join("renamed_file2.txt").exists());
        assert!(!dir.join("file1.txt").exists());
        assert!(!dir.join("file2.txt").exists());
        assert!(app.state.applied);
        assert!(app.state.results.iter().all(|r| r.success));

        // Verify undo log was created
        assert!(undo_path.exists());
        let log_content = fs::read_to_string(&undo_path).unwrap();
        let log: nomforge_core::UndoLog = serde_json::from_str(&log_content).unwrap();
        assert_eq!(log.batches.len(), 1);
        assert_eq!(log.batches[0].operations.len(), 2);

        // Perform undo
        app.perform_undo();

        // Verify files are restored
        assert!(dir.join("file1.txt").exists());
        assert!(dir.join("file2.txt").exists());
        assert!(!dir.join("renamed_file1.txt").exists());
        assert!(!dir.join("renamed_file2.txt").exists());
    }

    #[test]
    fn do_apply_skips_undo_when_disabled() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();

        fs::write(dir.join("file1.txt"), "content").unwrap();
        let undo_path = dir.join("undo_log.json");

        let mut app = NomforgeApp {
            state: State {
                dir: dir.to_string_lossy().into_owned(),
                prefix: "test_".into(),
                no_undo: true,
                history_file: undo_path.to_string_lossy().into_owned(),
                ..Default::default()
            },
        };

        app.scan_files();
        let rules = app.state.build_rules().unwrap();
        let engine = nomforge_core::RenameEngine::new(rules);
        let plans = engine.plan(&app.state.files).unwrap();
        let files = app.state.files.clone();

        app.do_apply(plans, files);

        assert!(dir.join("test_file1.txt").exists());
        // Undo log should not be created when no_undo is true
        assert!(!undo_path.exists());
    }
}
