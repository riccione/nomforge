use std::borrow::Cow;
use std::path::Path;

use eframe::egui;

use crate::state::State;

/// File name for display, borrowing when the path is valid UTF-8.
fn display_name(path: &Path) -> Cow<'_, str> {
    path.file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default()
}

/// Render the preview table panel showing plans and/or results.
pub fn show(ui: &mut egui::Ui, state: &State) {
    // Show conflicts if any
    if !state.pending_conflicts.is_empty() {
        ui.colored_label(
            egui::Color32::from_rgb(255, 165, 0),
            format!(
                "{} conflict(s) detected — files may be overwritten",
                state.pending_conflicts.len()
            ),
        );
        for conflict in &state.pending_conflicts {
            ui.label(
                egui::RichText::new(format!("  • {}", conflict.reason))
                    .color(egui::Color32::from_rgb(255, 165, 0)),
            );
        }
        ui.separator();
    }

    // Show plans (preview mode)
    if !state.plans.is_empty() {
        ui.heading("Preview");
        egui::ScrollArea::vertical()
            .id_salt("preview_scroll")
            .show(ui, |ui| {
                egui::Grid::new("preview_grid")
                    .striped(true)
                    .num_columns(3)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("FILE").strong());
                        ui.label(egui::RichText::new("NEW NAME").strong());
                        ui.label(egui::RichText::new("").strong());
                        ui.end_row();

                        for plan in &state.plans {
                            let source = display_name(&plan.source);
                            let target = display_name(&plan.target);

                            if plan.source == plan.target {
                                ui.label(source);
                                ui.label(egui::RichText::new("(no change)").weak());
                                ui.label("");
                            } else {
                                ui.label(source);
                                ui.label(target);
                                ui.label(egui::RichText::new("->").weak());
                            }
                            ui.end_row();
                        }
                    });
            });
    }

    // Show results (after apply)
    if !state.results.is_empty() {
        ui.heading("Results");
        egui::ScrollArea::vertical()
            .id_salt("results_scroll")
            .show(ui, |ui| {
                egui::Grid::new("results_grid")
                    .striped(true)
                    .num_columns(3)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("FILE").strong());
                        ui.label(egui::RichText::new("NEW NAME").strong());
                        ui.label(egui::RichText::new("STATUS").strong());
                        ui.end_row();

                        for result in &state.results {
                            let source = display_name(&result.source);
                            let target = display_name(&result.target);

                            if result.source == result.target {
                                ui.label(source);
                                ui.label(egui::RichText::new("(no change)").weak());
                                ui.label(egui::RichText::new("unchanged").weak());
                            } else if result.success {
                                ui.label(source);
                                ui.label(target);
                                ui.label(
                                    egui::RichText::new("ok")
                                        .strong()
                                        .color(egui::Color32::GREEN),
                                );
                            } else {
                                ui.label(source);
                                ui.label(target);
                                ui.label(
                                    egui::RichText::new("FAILED")
                                        .strong()
                                        .color(egui::Color32::RED),
                                );
                            }
                            ui.end_row();
                        }
                    });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn display_name_borrows_valid_utf8() {
        let path = PathBuf::from("/tmp/photos/photo.jpg");
        let name = display_name(&path);
        assert_eq!(name, "photo.jpg");
        assert!(matches!(name, Cow::Borrowed(_)));
    }

    #[test]
    fn display_name_without_file_name_is_empty() {
        assert_eq!(display_name(Path::new("/")), "");
    }

    #[cfg(unix)]
    #[test]
    fn display_name_lossy_for_non_utf8() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let path = PathBuf::from(OsStr::from_bytes(b"bad\xFF.txt"));
        let name = display_name(&path);
        assert!(name.contains('\u{FFFD}'), "name: {name:?}");
    }

    #[test]
    fn preview_and_results_tables_do_not_clash() {
        let ctx = egui::Context::default();
        let state = State {
            plans: vec![nomforge_core::RenamePlan {
                source: PathBuf::from("/tmp/a.txt"),
                target: PathBuf::from("/tmp/b.txt"),
            }],
            results: vec![nomforge_core::RenameResult {
                source: PathBuf::from("/tmp/a.txt"),
                target: PathBuf::from("/tmp/b.txt"),
                success: true,
                error: None,
            }],
            ..Default::default()
        };

        // Both tables render in the same pass after an apply; egui paints
        // ID-clash warnings as debug text into the frame output.
        let output = ctx.run_ui(egui::RawInput::default(), |ui| show(ui, &state));
        let clash = output.shapes.iter().any(|clipped| {
            matches!(
                &clipped.shape,
                egui::epaint::Shape::Text(text) if text.galley.job.text.contains("First use of")
            )
        });
        output.drop_without_applying_deltas();

        assert!(!clash, "preview and results tables share widget IDs");
    }
}
