pub mod folder_picker;
pub mod preview_table;
pub mod rule_builder;
pub mod settings;
pub mod status_bar;

/// Whether `response` was submitted with Enter.
pub fn submitted(ui: &eframe::egui::Ui, response: &eframe::egui::Response) -> bool {
    response.lost_focus() && ui.input(|i| i.key_pressed(eframe::egui::Key::Enter))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui;

    #[test]
    fn submitted_detects_enter_in_text_edit() {
        let ctx = egui::Context::default();
        let mut text = String::new();

        // Lay the editor out once to obtain a stable id, then focus it.
        let mut id = egui::Id::NULL;
        ctx.run_ui(Default::default(), |ui| {
            id = ui.text_edit_singleline(&mut text).id;
        })
        .drop_without_applying_deltas();
        ctx.memory_mut(|memory| memory.request_focus(id));

        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });

        let mut submitted_now = false;
        ctx.run_ui(input, |ui| {
            let response = ui.text_edit_singleline(&mut text);
            submitted_now = submitted(ui, &response);
        })
        .drop_without_applying_deltas();

        assert!(submitted_now, "Enter in a text edit should submit");
    }
}
