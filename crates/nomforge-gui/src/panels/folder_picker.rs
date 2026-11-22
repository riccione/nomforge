use eframe::egui;

use crate::state::State;

/// Render the folder picker panel.
///
/// `rfd::FileDialog` is a native modal dialog and blocks by design; rfd
/// recommends running dialogs on the main thread, so this stays synchronous.
///
/// Returns `true` when the user picked a directory (native dialog) or
/// confirmed the path field with Enter.
pub fn show(ui: &mut egui::Ui, state: &mut State) -> bool {
    let mut picked = false;

    ui.horizontal(|ui| {
        ui.label("Directory:");
        let path_edit = ui.text_edit_singleline(&mut state.dir);
        if path_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            picked = true;
        }
        if ui.button("Browse...").clicked()
            && let Some(path) = rfd::FileDialog::new().pick_folder()
        {
            state.dir = path.display().to_string();
            picked = true;
        }
    });

    picked
}

#[cfg(test)]
mod tests {
    #[test]
    fn folder_picker_panel_compiles() {
        // Module compiles and is usable
    }
}
