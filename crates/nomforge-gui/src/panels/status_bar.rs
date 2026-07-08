use eframe::egui;

use crate::state::State;
use crate::widgets::conflict_badge;

/// Render the status bar panel.
pub fn show(ui: &mut egui::Ui, state: &State) {
    ui.separator();

    ui.horizontal(|ui| {
        // Status message with color based on content
        let status_text = &state.status;
        let colored_text = if status_text.contains("error")
            || status_text.contains("Error")
            || status_text.contains("FAILED")
        {
            egui::RichText::new(status_text).color(egui::Color32::RED)
        } else if status_text.contains("Found")
            || status_text.contains("ok")
            || status_text.contains("Renamed")
        {
            egui::RichText::new(status_text).color(egui::Color32::GREEN)
        } else if status_text.contains("Ready") {
            egui::RichText::new(status_text).color(egui::Color32::GRAY)
        } else {
            egui::RichText::new(status_text)
        };
        ui.label(colored_text);

        // Show conflict badge if there are pending conflicts
        if !state.pending_conflicts.is_empty() {
            conflict_badge::show_conflict_count(ui, state.pending_conflicts.len());
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Summary info
            let file_count = state.files.len();
            let plan_count = state.plans.len();
            let result_count = state.results.len();

            if result_count > 0 {
                let succeeded = state.results.iter().filter(|r| r.success).count();
                let failed = result_count - succeeded;
                if failed > 0 {
                    ui.label(
                        egui::RichText::new(format!("{succeeded} ok, {failed} failed"))
                            .color(egui::Color32::RED),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(format!("{succeeded} ok")).color(egui::Color32::GREEN),
                    );
                }
            } else if plan_count > 0 {
                ui.label(format!("{plan_count} planned"));
            } else if file_count > 0 {
                ui.label(format!("{file_count} files"));
            }
        });
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn status_bar_panel_compiles() {
        // Module compiles and is usable
    }
}
