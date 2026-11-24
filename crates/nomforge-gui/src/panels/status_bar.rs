use eframe::egui;

use crate::state::State;
use crate::widgets::conflict_badge;

/// Color for a status message based on its content.
fn status_color(status: &str) -> Option<egui::Color32> {
    if status.contains("error") || status.contains("Error") || status.contains("FAILED") {
        Some(egui::Color32::RED)
    } else if status.contains("Found") || status.contains("ok") || status.contains("Renamed") {
        Some(egui::Color32::GREEN)
    } else if status.contains("Ready") {
        Some(egui::Color32::GRAY)
    } else {
        None
    }
}

/// Right-side summary text and optional color.
fn summary(state: &State) -> Option<(String, Option<egui::Color32>)> {
    if !state.results.is_empty() {
        let succeeded = state.results.iter().filter(|r| r.success).count();
        let failed = state.results.len() - succeeded;
        if failed > 0 {
            Some((
                format!("{succeeded} ok, {failed} failed"),
                Some(egui::Color32::RED),
            ))
        } else {
            Some((format!("{succeeded} ok"), Some(egui::Color32::GREEN)))
        }
    } else if !state.plans.is_empty() {
        Some((format!("{} planned", state.plans.len()), None))
    } else if !state.files.is_empty() {
        Some((format!("{} files", state.files.len()), None))
    } else {
        None
    }
}

/// Render the status bar panel.
pub fn show(ui: &mut egui::Ui, state: &State) {
    ui.separator();

    ui.horizontal(|ui| {
        // Status message with color based on content
        let mut status = egui::RichText::new(&state.status);
        if let Some(color) = status_color(&state.status) {
            status = status.color(color);
        }
        ui.label(status);

        // Show conflict badge if there are pending conflicts
        if !state.pending_conflicts.is_empty() {
            conflict_badge::show_conflict_count(ui, state.pending_conflicts.len());
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some((text, color)) = summary(state) {
                let mut rich = egui::RichText::new(text);
                if let Some(color) = color {
                    rich = rich.color(color);
                }
                ui.label(rich);
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use nomforge_core::{RenamePlan, RenameResult};
    use std::path::PathBuf;

    fn plan() -> RenamePlan {
        RenamePlan {
            source: PathBuf::from("/tmp/a.txt"),
            target: PathBuf::from("/tmp/b.txt"),
        }
    }

    fn result(success: bool) -> RenameResult {
        RenameResult {
            source: PathBuf::from("/tmp/a.txt"),
            target: PathBuf::from("/tmp/b.txt"),
            success,
            error: if success { None } else { Some("denied".into()) },
        }
    }

    #[test]
    fn status_color_matches_message_kind() {
        assert_eq!(status_color("Scan error: denied"), Some(egui::Color32::RED));
        assert_eq!(status_color("FAILED to rename"), Some(egui::Color32::RED));
        assert_eq!(status_color("Found 3 file(s)"), Some(egui::Color32::GREEN));
        assert_eq!(
            status_color("Renamed 2 file(s)"),
            Some(egui::Color32::GREEN)
        );
        assert_eq!(status_color("Ready"), Some(egui::Color32::GRAY));
        assert_eq!(status_color("Planning…"), None);
    }

    #[test]
    fn summary_prefers_results_then_plans_then_files() {
        let mut state = State::default();
        assert_eq!(summary(&state), None);

        state.files = vec![PathBuf::from("/tmp/a.txt")];
        assert_eq!(
            summary(&state).map(|(text, _)| text),
            Some("1 files".into())
        );

        state.plans = vec![plan()];
        let (text, color) = summary(&state).unwrap();
        assert_eq!(text, "1 planned");
        assert_eq!(color, None);

        state.results = vec![result(true)];
        let (text, color) = summary(&state).unwrap();
        assert_eq!(text, "1 ok");
        assert_eq!(color, Some(egui::Color32::GREEN));

        state.results.push(result(false));
        let (text, color) = summary(&state).unwrap();
        assert_eq!(text, "1 ok, 1 failed");
        assert_eq!(color, Some(egui::Color32::RED));
    }
}
