use eframe::egui;

use crate::state::State;

/// What caused the directory to change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirPick {
    /// A folder was chosen in the native dialog.
    Dialog,
    /// The path field was confirmed with Enter.
    Enter,
}

/// Render the folder picker panel.
///
/// `rfd::FileDialog` is a native modal dialog and blocks by design; rfd
/// recommends running dialogs on the main thread, so this stays synchronous.
///
/// Returns how the directory was picked, if it changed.
pub fn show(ui: &mut egui::Ui, state: &mut State) -> Option<DirPick> {
    let mut pick = None;

    ui.horizontal(|ui| {
        ui.label("Directory:");
        let path_edit = ui.text_edit_singleline(&mut state.dir);
        if crate::panels::submitted(ui, &path_edit) {
            pick = Some(DirPick::Enter);
        }
        if ui.button("Browse...").clicked()
            && let Some(path) = rfd::FileDialog::new().pick_folder()
        {
            state.dir = path.display().to_string();
            pick = Some(DirPick::Dialog);
        }
    });

    pick
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_without_interaction_returns_none() {
        let ctx = egui::Context::default();
        let mut state = State::default();
        let mut pick = Some(DirPick::Dialog);

        ctx.run_ui(Default::default(), |ui| {
            pick = show(ui, &mut state);
        })
        .drop_without_applying_deltas();

        assert_eq!(pick, None);
    }
}
