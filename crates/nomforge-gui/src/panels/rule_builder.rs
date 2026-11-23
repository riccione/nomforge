use eframe::egui;

use crate::state::State;

/// Render the rule builder panel (simple mode by default, regex in advanced mode).
///
/// Returns `true` when a field was submitted with Enter, which the caller uses
/// to refresh the preview.
pub fn show(ui: &mut egui::Ui, state: &mut State) -> bool {
    let mut submitted = false;

    ui.collapsing("Rules", |ui| {
        ui.horizontal(|ui| {
            ui.label("Find:");
            let response = ui.text_edit_singleline(&mut state.find);
            submitted |= crate::panels::submitted(ui, &response);
        });
        ui.horizontal(|ui| {
            ui.label("Replace:");
            let response = ui.text_edit_singleline(&mut state.replace);
            submitted |= crate::panels::submitted(ui, &response);
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Prefix:");
            let response = ui.text_edit_singleline(&mut state.prefix);
            submitted |= crate::panels::submitted(ui, &response);
        });
        ui.horizontal(|ui| {
            ui.label("Suffix:");
            let response = ui.text_edit_singleline(&mut state.suffix);
            submitted |= crate::panels::submitted(ui, &response);
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Remove:");
            let response = ui.text_edit_singleline(&mut state.remove);
            submitted |= crate::panels::submitted(ui, &response);
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Case:");
            egui::ComboBox::from_id_salt("case_combo")
                .selected_text(&state.case)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.case, String::new(), "(none)");
                    ui.selectable_value(&mut state.case, "upper".into(), "Upper");
                    ui.selectable_value(&mut state.case, "lower".into(), "Lower");
                    ui.selectable_value(&mut state.case, "title".into(), "Title");
                });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Change Extension (without dot):");
            let response = ui.text_edit_singleline(&mut state.ext_change);
            submitted |= crate::panels::submitted(ui, &response);
        });

        // Advanced mode: regex and counter fields
        ui.separator();
        ui.checkbox(&mut state.advanced_mode, "Advanced mode");
        if state.advanced_mode {
            ui.horizontal(|ui| {
                ui.label("Regex:");
                let response = ui.text_edit_singleline(&mut state.regex);
                submitted |= crate::panels::submitted(ui, &response);
            });
            ui.horizontal(|ui| {
                ui.label("Replacement:");
                let response = ui.text_edit_singleline(&mut state.replacement);
                submitted |= crate::panels::submitted(ui, &response);
            });

            ui.separator();

            ui.label("Counter:");
            ui.horizontal(|ui| {
                ui.label("Start:");
                let response =
                    ui.add(egui::DragValue::new(&mut state.counter_start).range(0..=10000));
                submitted |= crate::panels::submitted(ui, &response);
            });
            ui.horizontal(|ui| {
                ui.label("Padding:");
                let response =
                    ui.add(egui::DragValue::new(&mut state.counter_padding).range(0..=10));
                submitted |= crate::panels::submitted(ui, &response);
            });
            ui.horizontal(|ui| {
                ui.label("Position:");
                egui::ComboBox::from_id_salt("counter_position")
                    .selected_text(&state.counter_position)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.counter_position, "prefix".into(), "Prefix");
                        ui.selectable_value(&mut state.counter_position, "suffix".into(), "Suffix");
                        ui.selectable_value(
                            &mut state.counter_position,
                            "replace".into(),
                            "Replace stem",
                        );
                    });
            });
        }
    });

    submitted
}

#[cfg(test)]
mod tests {
    #[test]
    fn rule_builder_panel_compiles() {
        // Module compiles and is usable
    }
}
