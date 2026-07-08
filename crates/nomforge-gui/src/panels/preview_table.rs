use eframe::egui;

use crate::state::State;

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
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("preview_grid")
                .striped(true)
                .num_columns(3)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("FILE").strong());
                    ui.label(egui::RichText::new("NEW NAME").strong());
                    ui.label(egui::RichText::new("").strong());
                    ui.end_row();

                    for plan in &state.plans {
                        let source = plan
                            .source
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        let target = plan
                            .target
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default();

                        if plan.source == plan.target {
                            ui.label(&source);
                            ui.label(egui::RichText::new("(no change)").weak());
                            ui.label("");
                        } else {
                            ui.label(&source);
                            ui.label(&target);
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
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("results_grid")
                .striped(true)
                .num_columns(3)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("FILE").strong());
                    ui.label(egui::RichText::new("NEW NAME").strong());
                    ui.label(egui::RichText::new("STATUS").strong());
                    ui.end_row();

                    for result in &state.results {
                        let source = result
                            .source
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        let target = result
                            .target
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default();

                        if result.source == result.target {
                            ui.label(&source);
                            ui.label(egui::RichText::new("(no change)").weak());
                            ui.label(egui::RichText::new("unchanged").weak());
                        } else if result.success {
                            ui.label(&source);
                            ui.label(&target);
                            ui.label(
                                egui::RichText::new("ok")
                                    .strong()
                                    .color(egui::Color32::GREEN),
                            );
                        } else {
                            ui.label(&source);
                            ui.label(&target);
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
    #[test]
    fn preview_table_panel_compiles() {
        // Module compiles and is usable
    }
}
