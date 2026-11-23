use std::path::PathBuf;

use eframe::egui;
use nomforge_core::{RenameRule, ScanOptions};

use crate::job::{self, Job, JobOutcome};
use crate::panels::folder_picker::DirPick;
use crate::state::State;

/// Suffix noting entries that could not be read during a scan.
fn skipped_suffix(count: usize) -> String {
    if count == 0 {
        String::new()
    } else {
        format!(" ({count} unreadable entries skipped)")
    }
}

/// The nomforge GUI application.
#[derive(Default)]
pub struct NomforgeApp {
    state: State,
}

/// Inputs captured on the UI thread for a planning job.
struct PlanJobInput {
    dir: PathBuf,
    options: ScanOptions,
    rules: Vec<RenameRule>,
}

impl eframe::App for NomforgeApp {
    /// The root UI has no background of its own, so the window clear color is
    /// what shows through. Derive it from the active theme's window fill so
    /// switching between light and dark restyles the whole background.
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.window_fill().to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_job(ui.ctx());

        ui.horizontal(|ui| {
            ui.heading(format!("nomforge v{}", { env!("CARGO_PKG_VERSION") }));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Settings").clicked() {
                    self.state.open_settings();
                }
            });
        });
        ui.separator();

        // Directory picker; Enter always previews, a dialog follows auto-scan.
        let dir_pick = crate::panels::folder_picker::show(ui, &mut self.state);
        self.handle_dir_pick(ui.ctx(), dir_pick);

        ui.separator();

        // Rules section
        let mut preview_requested = crate::panels::rule_builder::show(ui, &mut self.state);

        // Filtering section
        ui.collapsing("Filtering", |ui| {
            ui.horizontal(|ui| {
                ui.label("Filter by extension:");
                let response = ui.text_edit_singleline(&mut self.state.filter_ext);
                preview_requested |= crate::panels::submitted(ui, &response);
            });
            ui.horizontal(|ui| {
                ui.label("Include pattern:");
                let response = ui.text_edit_singleline(&mut self.state.include);
                preview_requested |= crate::panels::submitted(ui, &response);
            });
            ui.horizontal(|ui| {
                ui.label("Exclude pattern:");
                let response = ui.text_edit_singleline(&mut self.state.exclude);
                preview_requested |= crate::panels::submitted(ui, &response);
            });
            ui.checkbox(&mut self.state.recursive, "Recursive");
            ui.checkbox(
                &mut self.state.hidden,
                "Include hidden files and directories",
            );
        });

        // Enter in any rule or filter field refreshes the preview.
        self.preview_on_submit(ui.ctx(), preview_requested);

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
            if ui
                .add_enabled(
                    self.state.job.is_none(),
                    egui::Button::new("Undo Last Batch"),
                )
                .clicked()
            {
                self.state.show_undo_modal = true;
            }
        });

        // Undo confirmation modal
        if self.state.show_undo_modal {
            let modal = egui::Modal::new(egui::Id::unique("undo_confirm")).show(ui.ctx(), |ui| {
                ui.set_width(300.0);
                ui.heading("Undo Last Batch?");
                ui.label("This will revert the most recent rename operation.");
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        self.perform_undo(ui.ctx());
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

        // Settings modal
        crate::panels::settings::show(ui, &mut self.state);

        // Conflict confirmation modal
        self.show_conflict_modal(ui);

        ui.separator();

        // Actions (disabled while a background job is running)
        ui.add_enabled_ui(self.state.job.is_none(), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Scan").clicked() {
                    self.scan_files(ui.ctx());
                }
                if ui.button("Preview").clicked() {
                    self.preview(ui.ctx());
                }
                if ui.button("Apply").clicked() {
                    self.apply(ui.ctx());
                }
            });
        });

        // Status bar
        crate::panels::status_bar::show(ui, &self.state);

        // Preview and results table
        crate::panels::preview_table::show(ui, &self.state);
    }
}

impl NomforgeApp {
    /// Create the app with the given state (settings already loaded).
    pub fn new(state: State) -> Self {
        Self { state }
    }

    /// Resolve the configured directory to an absolute path.
    ///
    /// Plans built from absolute paths keep undo log entries valid no matter
    /// which working directory a later undo runs from.
    fn resolve_dir(&mut self) -> Option<PathBuf> {
        match std::fs::canonicalize(&self.state.dir) {
            Ok(dir) => Some(dir),
            Err(e) => {
                self.state.status = format!("Cannot resolve directory {}: {e}", self.state.dir);
                None
            }
        }
    }

    /// Scan the selected directory when auto-scan is enabled and the user just
    /// picked one.
    fn auto_scan_if_enabled(&mut self, ctx: &egui::Context, dir_picked: bool) {
        if dir_picked && self.state.settings.auto_scan {
            self.scan_files(ctx);
        }
    }

    /// React to a directory pick: Enter always previews, while a dialog
    /// selection follows the auto-scan setting.
    fn handle_dir_pick(&mut self, ctx: &egui::Context, pick: Option<DirPick>) {
        match pick {
            None => {}
            Some(DirPick::Enter) => self.preview(ctx),
            Some(DirPick::Dialog) => self.auto_scan_if_enabled(ctx, true),
        }
    }

    /// Refresh the preview when a field was submitted with Enter.
    fn preview_on_submit(&mut self, ctx: &egui::Context, requested: bool) {
        if requested {
            self.preview(ctx);
        }
    }

    /// Whether a background job is currently running.
    fn is_busy(&self) -> bool {
        self.state.job.is_some()
    }

    /// Reset the output, resolve the directory and capture the scan options.
    ///
    /// Returns `None` (after setting the status) when a job is already running
    /// or the directory cannot be resolved.
    fn prepare_scan(&mut self, status: Option<&str>) -> Option<(PathBuf, ScanOptions)> {
        if self.is_busy() {
            return None;
        }

        // `reset_output` sets the status to "Ready"; preserve it when the
        // caller wants a quiet preparation (e.g. the post-undo refresh).
        let previous_status = self.state.status.clone();
        self.state.reset_output();

        let dir = self.resolve_dir()?;
        let options = self.state.build_scan_options();
        self.state.status = match status {
            Some(status) => status.into(),
            None => previous_status,
        };
        Some((dir, options))
    }

    /// Like [`Self::prepare_scan`], plus the rules needed for planning.
    fn prepare_plan(&mut self, status: Option<&str>) -> Option<PlanJobInput> {
        let (dir, options) = self.prepare_scan(status)?;

        let rules = match self.state.build_rules() {
            Ok(rules) => rules,
            Err(e) => {
                self.state.status = format!("Rule error: {e}");
                return None;
            }
        };

        Some(PlanJobInput {
            dir,
            options,
            rules,
        })
    }

    fn scan_files(&mut self, ctx: &egui::Context) {
        let Some((dir, options)) = self.prepare_scan(Some("Scanning\u{2026}")) else {
            return;
        };
        self.spawn(ctx, move || JobOutcome::Scan(job::scan(&dir, &options)));
    }

    fn preview(&mut self, ctx: &egui::Context) {
        let Some(work) = self.prepare_plan(Some("Planning\u{2026}")) else {
            return;
        };
        self.spawn(ctx, move || {
            JobOutcome::Preview(job::preview(&work.dir, &work.options, work.rules))
        });
    }

    fn apply(&mut self, ctx: &egui::Context) {
        let Some(work) = self.prepare_plan(Some("Applying\u{2026}")) else {
            return;
        };

        // Always rescan: filters and scan options may have changed since the
        // last scan, so reusing a cached file list could act on excluded files.
        let history = self.undo_log_target();
        self.spawn(ctx, move || {
            JobOutcome::Apply(job::apply(&work.dir, &work.options, work.rules, history))
        });
    }

    /// Refresh plans after an undo while keeping the undo confirmation visible.
    fn preview_after_undo(&mut self, ctx: &egui::Context) {
        let undo_status = self.state.status.clone();
        let Some(work) = self.prepare_plan(None) else {
            // Keep the confirmation even if the refresh cannot start.
            self.state.status = undo_status;
            return;
        };
        self.spawn_quiet(ctx, move || {
            JobOutcome::Preview(job::preview(&work.dir, &work.options, work.rules))
        });
    }

    fn perform_undo(&mut self, ctx: &egui::Context) {
        if self.is_busy() {
            return;
        }

        let history = self.history_path();
        self.state.status = "Undoing\u{2026}".into();
        self.spawn(ctx, move || JobOutcome::Undo(job::undo(&history)));
    }

    /// Configured undo-log path.
    fn history_path(&self) -> PathBuf {
        if self.state.history_file.is_empty() {
            nomforge_core::default_undo_log_path()
        } else {
            PathBuf::from(&self.state.history_file)
        }
    }

    /// Undo-log path, unless undo logging is disabled.
    fn undo_log_target(&self) -> Option<PathBuf> {
        (!self.state.no_undo).then(|| self.history_path())
    }

    /// Apply plans that the conflict modal confirmed.
    fn spawn_confirmed_apply(
        &mut self,
        plans: Vec<nomforge_core::RenamePlan>,
        ctx: &egui::Context,
    ) {
        if self.is_busy() {
            return;
        }

        let history = self.undo_log_target();
        self.state.status = "Applying\u{2026}".into();
        self.spawn(ctx, move || {
            JobOutcome::Apply(Ok(job::apply_confirmed(plans, history, 0)))
        });
    }

    /// Run `work` on a worker thread and repaint the UI when it finishes.
    fn spawn<F>(&mut self, ctx: &egui::Context, work: F)
    where
        F: FnOnce() -> JobOutcome + Send + 'static,
    {
        self.spawn_with(ctx, false, work);
    }

    /// Like [`Self::spawn`], but leaves the current status message untouched
    /// when a successful preview result arrives.
    fn spawn_quiet<F>(&mut self, ctx: &egui::Context, work: F)
    where
        F: FnOnce() -> JobOutcome + Send + 'static,
    {
        self.spawn_with(ctx, true, work);
    }

    fn spawn_with<F>(&mut self, ctx: &egui::Context, keep_status: bool, work: F)
    where
        F: FnOnce() -> JobOutcome + Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(work());
            ctx.request_repaint();
        });
        self.state.job = Some(Job { rx, keep_status });
    }

    /// Pick up the outcome of a finished background job, if any.
    fn poll_job(&mut self, ctx: &egui::Context) {
        let Some(job) = self.state.job.as_ref() else {
            return;
        };

        let outcome = match job.rx.try_recv() {
            Ok(outcome) => outcome,
            Err(std::sync::mpsc::TryRecvError::Empty) => return,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.state.job = None;
                self.state.status = "Background job failed".into();
                return;
            }
        };

        let keep_status = job.keep_status;
        self.state.job = None;
        self.finish(outcome, ctx, keep_status);
    }

    /// Apply a finished job's outcome to the UI state.
    fn finish(&mut self, outcome: JobOutcome, ctx: &egui::Context, keep_status: bool) {
        match outcome {
            JobOutcome::Scan(Ok((files, skipped))) => {
                self.state.status =
                    format!("Found {} file(s){}", files.len(), skipped_suffix(skipped));
                self.state.files = files.clone();

                // Switch to the preview screen by planning the scanned files,
                // without walking the directory a second time.
                match self.state.build_rules() {
                    Ok(rules) => {
                        self.spawn(ctx, move || {
                            JobOutcome::Preview(job::preview_files(files, rules, skipped))
                        });
                    }
                    Err(e) => self.state.status = format!("Rule error: {e}"),
                }
            }
            JobOutcome::Scan(Err(message)) => self.state.status = message,
            JobOutcome::Preview(Ok(data)) => {
                if data.plans.is_empty() {
                    if !keep_status {
                        self.state.status = "No files found".into();
                    }
                    return;
                }
                if !keep_status {
                    self.state.status = format!(
                        "{} file(s) to rename{}",
                        data.plans.len(),
                        skipped_suffix(data.skipped)
                    );
                }
                self.state.pending_conflicts = data.conflicts;
                self.state.plans = data.plans;
            }
            JobOutcome::Preview(Err(message)) => self.state.status = message,
            JobOutcome::Apply(Ok(data)) => {
                if data.plans.is_empty() {
                    self.state.status = "No files found".into();
                    return;
                }
                if !data.conflicts.is_empty() {
                    self.state.pending_conflicts = data.conflicts;
                    self.state.show_conflict_modal = true;
                    self.state.status = format!(
                        "{} conflict(s) detected \u{2014} review before applying{}",
                        self.state.pending_conflicts.len(),
                        skipped_suffix(data.skipped)
                    );
                    self.state.plans = data.plans;
                    return;
                }

                let renamed = data
                    .results
                    .iter()
                    .filter(|r| r.success && r.source != r.target)
                    .count();
                self.state.status = match data.undo_error {
                    Some(e) => format!("Renamed {renamed} file(s) (undo log error: {e})"),
                    None => format!("Renamed {renamed} file(s){}", skipped_suffix(data.skipped)),
                };
                self.state.plans = data.plans;
                self.state.results = data.results;
                self.state.applied = true;
            }
            JobOutcome::Apply(Err(message)) => self.state.status = message,
            JobOutcome::Undo(Ok((reverted, skipped))) => {
                self.state.status = if skipped > 0 {
                    format!("Reverted {reverted} file(s), skipped {skipped} existing")
                } else {
                    format!("Reverted {reverted} file(s)")
                };
                // Refresh the preview against the restored files without
                // overwriting the undo confirmation.
                self.preview_after_undo(ctx);
            }
            JobOutcome::Undo(Err(message)) => self.state.status = message,
        }
    }

    #[cfg(test)]
    fn wait_for_job(&mut self, ctx: &egui::Context) {
        let job = self.state.job.take().expect("no job to wait for");
        let outcome = job.rx.recv().expect("job thread dropped");
        self.finish(outcome, ctx, job.keep_status);
    }

    fn show_conflict_modal(&mut self, ui: &mut egui::Ui) {
        if !self.state.show_conflict_modal {
            return;
        }

        let modal = egui::Modal::new(egui::Id::unique("conflict_confirm")).show(ui.ctx(), |ui| {
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
            ui.label(
                "Apply anyway? Conflicting renames will fail — the first plan wins, nothing is overwritten.",
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Apply Anyway").clicked() {
                    let plans = std::mem::take(&mut self.state.plans);
                    self.state.show_conflict_modal = false;
                    self.state.pending_conflicts.clear();
                    self.spawn_confirmed_apply(plans, ui.ctx());
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
    fn clear_color_follows_theme() {
        let app = NomforgeApp::default();

        let light = eframe::App::clear_color(&app, &egui::Visuals::light());
        let dark = eframe::App::clear_color(&app, &egui::Visuals::dark());

        assert_eq!(
            light,
            egui::Visuals::light()
                .window_fill()
                .to_normalized_gamma_f32()
        );
        assert_eq!(
            dark,
            egui::Visuals::dark()
                .window_fill()
                .to_normalized_gamma_f32()
        );
        // Light theme must actually produce a brighter background.
        assert!(light[0] > dark[0], "light={light:?} dark={dark:?}");
    }

    #[test]
    fn do_apply_logs_renames_and_allows_undo() {
        let tmp = TempDir::new().unwrap();
        // Keep the undo log outside the scanned directory so the post-undo
        // refresh does not plan to rename it.
        let dir = tmp.path().join("files");
        fs::create_dir_all(&dir).unwrap();

        // Create test files
        fs::write(dir.join("file1.txt"), "content1").unwrap();
        fs::write(dir.join("file2.txt"), "content2").unwrap();

        // Set up undo log path
        let undo_path = tmp.path().join("undo_log.json");

        let mut app = NomforgeApp {
            state: State {
                dir: dir.to_string_lossy().into_owned(),
                prefix: "renamed_".into(),
                history_file: undo_path.to_string_lossy().into_owned(),
                ..Default::default()
            },
        };

        let ctx = egui::Context::default();

        // Scan files
        app.scan_files(&ctx);
        app.wait_for_job(&ctx); // scan
        app.wait_for_job(&ctx); // automatic preview
        assert_eq!(app.state.files.len(), 2);

        // Build rules and plans
        let rules = app.state.build_rules().unwrap();
        let engine = nomforge_core::RenameEngine::new(rules);
        let plans = engine.plan(&app.state.files).unwrap();
        assert_eq!(plans.len(), 2);

        // Apply renames with undo logging
        app.spawn_confirmed_apply(plans, &ctx);
        app.wait_for_job(&ctx);

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

        // Perform undo (the outcome chains a preview job)
        app.perform_undo(&ctx);
        app.wait_for_job(&ctx);
        app.wait_for_job(&ctx);

        // The undo confirmation survives the refresh job, which still
        // repopulates the plans.
        assert_eq!(app.state.status, "Reverted 2 file(s)");
        assert_eq!(app.state.plans.len(), 2);

        // Verify files are restored
        assert!(dir.join("file1.txt").exists());
        assert!(dir.join("file2.txt").exists());
        assert!(!dir.join("renamed_file1.txt").exists());
        assert!(!dir.join("renamed_file2.txt").exists());
    }

    #[test]
    fn preview_rescans_with_current_filters() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("a.txt"), "a").unwrap();
        fs::write(dir.join("b.jpg"), "b").unwrap();

        let mut app = NomforgeApp {
            state: State {
                dir: dir.to_string_lossy().into_owned(),
                prefix: "x_".into(),
                ..Default::default()
            },
        };

        let ctx = egui::Context::default();

        // Cache a scan that includes both files, then tighten the filter.
        app.scan_files(&ctx);
        app.wait_for_job(&ctx); // scan
        app.wait_for_job(&ctx); // automatic preview
        assert_eq!(app.state.files.len(), 2);
        app.state.filter_ext = "txt".into();

        app.preview(&ctx);
        app.wait_for_job(&ctx);

        assert_eq!(app.state.plans.len(), 1);
        assert_eq!(app.state.plans[0].source.file_name().unwrap(), "a.txt");
        // A user-triggered preview still updates the status normally.
        assert_eq!(app.state.status, "1 file(s) to rename");
    }

    #[test]
    fn apply_rescans_with_current_filters() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("a.txt"), "a").unwrap();
        fs::write(dir.join("b.jpg"), "b").unwrap();
        let undo_path = dir.join("undo_log.json");

        let mut app = NomforgeApp {
            state: State {
                dir: dir.to_string_lossy().into_owned(),
                prefix: "x_".into(),
                history_file: undo_path.to_string_lossy().into_owned(),
                ..Default::default()
            },
        };

        let ctx = egui::Context::default();

        app.scan_files(&ctx);
        app.wait_for_job(&ctx); // scan
        app.wait_for_job(&ctx); // automatic preview
        assert_eq!(app.state.files.len(), 2);
        app.state.filter_ext = "txt".into();

        app.apply(&ctx);
        app.wait_for_job(&ctx);

        assert!(dir.join("x_a.txt").exists());
        assert!(
            dir.join("b.jpg").exists(),
            "excluded file must not be renamed"
        );
    }

    #[test]
    fn preview_after_apply_uses_renamed_files() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("files");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.txt"), "a").unwrap();
        let undo_path = tmp.path().join("undo.json");

        let mut app = NomforgeApp {
            state: State {
                dir: dir.to_string_lossy().into_owned(),
                prefix: "x_".into(),
                history_file: undo_path.to_string_lossy().into_owned(),
                ..Default::default()
            },
        };

        let ctx = egui::Context::default();

        app.apply(&ctx);
        app.wait_for_job(&ctx);
        assert!(dir.join("x_a.txt").exists());
        assert!(
            app.state.files.is_empty(),
            "apply must not keep pre-rename paths"
        );

        // Previewing again rescans, so plans are based on the renamed file.
        app.preview(&ctx);
        app.wait_for_job(&ctx);
        assert_eq!(app.state.plans.len(), 1);
        assert_eq!(app.state.plans[0].source.file_name().unwrap(), "x_a.txt");
    }

    #[test]
    fn apply_clears_stale_conflicts() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("a.txt"), "content").unwrap();
        let undo_path = dir.join("undo_log.json");

        let mut app = NomforgeApp {
            state: State {
                dir: dir.to_string_lossy().into_owned(),
                prefix: "x_".into(),
                history_file: undo_path.to_string_lossy().into_owned(),
                // Left over from an earlier preview that had conflicts.
                pending_conflicts: vec![nomforge_core::Conflict {
                    plan_a: 0,
                    plan_b: 1,
                    reason: nomforge_core::ConflictReason::SameTarget(dir.join("dup.txt")),
                }],
                show_conflict_modal: true,
                ..Default::default()
            },
        };

        let ctx = egui::Context::default();

        // A clean rule set: applying it must drop the stale conflict state.
        app.apply(&ctx);
        app.wait_for_job(&ctx);

        assert!(dir.join("x_a.txt").exists());
        assert_eq!(app.state.status, "Renamed 1 file(s)");
        assert!(app.state.pending_conflicts.is_empty());
        assert!(!app.state.show_conflict_modal);
    }

    #[test]
    fn preview_on_submit_refreshes_only_when_requested() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("a.txt"), "a").unwrap();
        let ctx = egui::Context::default();

        let mut app = NomforgeApp::default();
        app.state.dir = dir.to_string_lossy().into_owned();
        app.state.prefix = "x_".into();

        app.preview_on_submit(&ctx, false);
        assert!(app.state.job.is_none());

        app.preview_on_submit(&ctx, true);
        assert!(app.state.job.is_some());
        app.wait_for_job(&ctx);
        assert_eq!(app.state.plans.len(), 1);
        assert_eq!(app.state.plans[0].target.file_name().unwrap(), "x_a.txt");
    }

    #[test]
    fn directory_enter_previews_without_auto_scan() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("a.txt"), "a").unwrap();
        let ctx = egui::Context::default();

        let mut app = NomforgeApp::default();
        app.state.dir = dir.to_string_lossy().into_owned();
        app.state.settings.auto_scan = false;

        app.handle_dir_pick(&ctx, Some(DirPick::Enter));
        assert!(app.state.job.is_some());
        app.wait_for_job(&ctx);
        assert_eq!(app.state.plans.len(), 1);
    }

    #[test]
    fn directory_dialog_respects_auto_scan() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("a.txt"), "a").unwrap();
        let ctx = egui::Context::default();

        let mut app = NomforgeApp::default();
        app.state.dir = dir.to_string_lossy().into_owned();
        app.state.settings.auto_scan = false;

        // Auto-scan off: choosing a folder does not start anything.
        app.handle_dir_pick(&ctx, Some(DirPick::Dialog));
        assert!(app.state.job.is_none());

        // Auto-scan on: scan, then the chained preview.
        app.state.settings.auto_scan = true;
        app.handle_dir_pick(&ctx, Some(DirPick::Dialog));
        assert!(app.state.job.is_some());
        app.wait_for_job(&ctx); // scan
        app.wait_for_job(&ctx); // automatic preview
        assert_eq!(app.state.plans.len(), 1);
    }

    #[test]
    fn auto_scan_only_when_enabled_and_picked() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("a.txt"), "a").unwrap();
        let ctx = egui::Context::default();
        let dir_str = dir.to_string_lossy().into_owned();

        // Enabled (default) + picked -> scans.
        let mut app = NomforgeApp::default();
        app.state.dir = dir_str.clone();
        app.auto_scan_if_enabled(&ctx, true);
        assert!(app.state.job.is_some());
        app.wait_for_job(&ctx); // scan
        app.wait_for_job(&ctx); // automatic preview
        assert_eq!(app.state.files.len(), 1);
        assert_eq!(app.state.plans.len(), 1);
        assert_eq!(app.state.status, "1 file(s) to rename");

        // Disabled -> nothing happens.
        let mut app = NomforgeApp::default();
        app.state.dir = dir_str.clone();
        app.state.settings.auto_scan = false;
        app.auto_scan_if_enabled(&ctx, true);
        assert!(app.state.job.is_none());
        assert_eq!(app.state.status, "Ready");

        // Nothing picked -> nothing happens.
        let mut app = NomforgeApp::default();
        app.state.dir = dir_str;
        app.auto_scan_if_enabled(&ctx, false);
        assert!(app.state.job.is_none());
        assert_eq!(app.state.status, "Ready");
    }

    #[test]
    fn scan_switches_to_preview() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("a.txt"), "a").unwrap();
        let undo_path = tmp.path().join("undo.json");

        let mut app = NomforgeApp::default();
        app.state.dir = dir.to_string_lossy().into_owned();
        app.state.prefix = "x_".into();
        app.state.history_file = undo_path.to_string_lossy().into_owned();

        let ctx = egui::Context::default();
        app.scan_files(&ctx);
        app.wait_for_job(&ctx); // scan
        app.wait_for_job(&ctx); // automatic preview

        assert_eq!(app.state.plans.len(), 1);
        assert_eq!(app.state.plans[0].source.file_name().unwrap(), "a.txt");
        assert_eq!(app.state.plans[0].target.file_name().unwrap(), "x_a.txt");
        assert_eq!(app.state.status, "1 file(s) to rename");
        assert!(app.state.pending_conflicts.is_empty());
    }

    #[test]
    fn scan_with_invalid_rules_lists_files_but_reports_rule_error() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("a.txt"), "a").unwrap();

        let mut app = NomforgeApp::default();
        app.state.dir = dir.to_string_lossy().into_owned();
        app.state.case = "sideways".into();

        let ctx = egui::Context::default();
        app.scan_files(&ctx);
        app.wait_for_job(&ctx);

        // No preview job is chained when the rules cannot be built.
        assert!(app.state.job.is_none());
        assert_eq!(app.state.files.len(), 1);
        assert!(
            app.state.status.starts_with("Rule error"),
            "status: {}",
            app.state.status
        );
    }

    #[test]
    fn preview_and_apply_report_rule_errors() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("a.txt"), "a").unwrap();
        let ctx = egui::Context::default();

        for action in ["preview", "apply"] {
            let mut app = NomforgeApp {
                state: State {
                    dir: dir.to_string_lossy().into_owned(),
                    // An unknown case transform makes `build_rules` fail.
                    case: "sideways".into(),
                    ..Default::default()
                },
            };

            if action == "preview" {
                app.preview(&ctx);
            } else {
                app.apply(&ctx);
            }

            assert!(app.state.job.is_none(), "{action} spawned a job");
            assert!(
                app.state.status.starts_with("Rule error"),
                "{action} status: {}",
                app.state.status
            );
            assert!(dir.join("a.txt").exists(), "{action} renamed a file");
        }
    }

    #[test]
    fn renamed_count_excludes_unchanged() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("a.txt"), "a").unwrap();
        fs::write(dir.join("keep.txt"), "k").unwrap();
        let undo_path = tmp.path().join("undo.json");

        let mut app = NomforgeApp {
            state: State {
                dir: dir.to_string_lossy().into_owned(),
                prefix: "x_".into(),
                history_file: undo_path.to_string_lossy().into_owned(),
                ..Default::default()
            },
        };

        let ctx = egui::Context::default();
        let plans = vec![
            nomforge_core::RenamePlan {
                source: dir.join("a.txt"),
                target: dir.join("x_a.txt"),
            },
            // No-op plan: must not count as renamed.
            nomforge_core::RenamePlan {
                source: dir.join("keep.txt"),
                target: dir.join("keep.txt"),
            },
        ];
        app.spawn_confirmed_apply(plans, &ctx);
        app.wait_for_job(&ctx);

        assert_eq!(app.state.status, "Renamed 1 file(s)");
    }

    #[test]
    fn do_apply_works_with_invalid_rule_text() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("a.txt"), "content").unwrap();
        let undo_path = dir.join("undo_log.json");

        let mut app = NomforgeApp {
            state: State {
                dir: dir.to_string_lossy().into_owned(),
                // Invalid regex: `build_rules` would fail, but applying the
                // already-computed plans must not consult the rules.
                regex: "[invalid".into(),
                history_file: undo_path.to_string_lossy().into_owned(),
                ..Default::default()
            },
        };

        let plans = vec![nomforge_core::RenamePlan {
            source: dir.join("a.txt"),
            target: dir.join("b.txt"),
        }];
        let ctx = egui::Context::default();
        app.spawn_confirmed_apply(plans, &ctx);
        app.wait_for_job(&ctx);

        assert!(dir.join("b.txt").exists());
        assert_eq!(app.state.status, "Renamed 1 file(s)");
        assert_eq!(nomforge_core::undo_count(&undo_path).unwrap(), 1);
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

        let ctx = egui::Context::default();
        app.scan_files(&ctx);
        app.wait_for_job(&ctx); // scan
        app.wait_for_job(&ctx); // automatic preview
        let rules = app.state.build_rules().unwrap();
        let engine = nomforge_core::RenameEngine::new(rules);
        let plans = engine.plan(&app.state.files).unwrap();

        app.spawn_confirmed_apply(plans, &ctx);
        app.wait_for_job(&ctx);

        assert!(dir.join("test_file1.txt").exists());
        // Undo log should not be created when no_undo is true
        assert!(!undo_path.exists());
    }
}
