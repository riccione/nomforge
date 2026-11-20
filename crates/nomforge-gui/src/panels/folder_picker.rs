use eframe::egui;

use crate::state::State;

/// Render the folder picker panel.
///
/// `rfd::FileDialog` is a native modal dialog and blocks by design; rfd
/// recommends running dialogs on the main thread, so this stays synchronous.
pub fn show(ui: &mut egui::Ui, state: &mut State) {
    ui.horizontal(|ui| {
        ui.label("Directory:");
        ui.text_edit_singleline(&mut state.dir);
        if ui.button("Browse...").clicked()
            && let Some(path) = rfd::FileDialog::new().pick_folder()
        {
            state.dir = path.display().to_string();
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn folder_picker_panel_compiles() {
        // Module compiles and is usable
    }
}
