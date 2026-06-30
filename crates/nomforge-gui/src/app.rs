use eframe::egui;

use crate::state::State;

/// The nomforge GUI application.
#[derive(Default)]
pub struct NomforgeApp {
    state: State,
}

impl eframe::App for NomforgeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("nomforge");
        ui.separator();

        // Directory picker
        crate::panels::folder_picker::show(ui, &mut self.state);

        ui.separator();

        // Rules section
        crate::panels::rule_builder::show(ui, &mut self.state);

        // Filtering section
        ui.collapsing("Filtering", |ui| {
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

        ui.separator();

        // Status
        ui.label(&self.state.status);

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

        let scan_options = self.state.build_scan_options();
        let files = match nomforge_core::scan_files(dir, &scan_options) {
            Ok(f) => f,
            Err(e) => {
                self.state.status = format!("Scan error: {e}");
                return;
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
        self.state.plans = plans;
    }

    fn apply(&mut self) {
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

        let scan_options = self.state.build_scan_options();
        let files = match nomforge_core::scan_files(dir, &scan_options) {
            Ok(f) => f,
            Err(e) => {
                self.state.status = format!("Scan error: {e}");
                return;
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

        let results = match engine.apply(&plans) {
            Ok(r) => r,
            Err(e) => {
                self.state.status = format!("Apply error: {e}");
                return;
            }
        };

        let succeeded = results.iter().filter(|r| r.success).count();
        self.state.status = format!("Renamed {succeeded} file(s)");
        self.state.results = results;
        self.state.applied = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_default_state() {
        let app = NomforgeApp::default();
        assert!(app.state.dir.is_empty());
        assert_eq!(app.state.status, "Ready");
    }
}
