use eframe::egui;

use crate::state::State;

/// Render the rule builder panel (simple mode by default, regex in advanced mode).
pub fn show(ui: &mut egui::Ui, state: &mut State) {
    ui.collapsing("Rules", |ui| {
        ui.horizontal(|ui| {
            ui.label("Find:");
            ui.text_edit_singleline(&mut state.find);
        });
        ui.horizontal(|ui| {
            ui.label("Replace:");
            ui.text_edit_singleline(&mut state.replace);
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Prefix:");
            ui.text_edit_singleline(&mut state.prefix);
        });
        ui.horizontal(|ui| {
            ui.label("Suffix:");
            ui.text_edit_singleline(&mut state.suffix);
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Remove:");
            ui.text_edit_singleline(&mut state.remove);
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
            ui.label("Change Extension:");
            ui.text_edit_singleline(&mut state.ext_change);
        });

        // Advanced mode: regex and counter fields
        ui.separator();
        ui.checkbox(&mut state.advanced_mode, "Advanced mode");
        if state.advanced_mode {
            ui.horizontal(|ui| {
                ui.label("Regex:");
                ui.text_edit_singleline(&mut state.regex);
            });
            ui.horizontal(|ui| {
                ui.label("Replacement:");
                ui.text_edit_singleline(&mut state.replacement);
            });

            ui.separator();

            ui.label("Counter:");
            ui.horizontal(|ui| {
                ui.label("Start:");
                ui.add(egui::DragValue::new(&mut state.counter_start).range(0..=10000));
            });
            ui.horizontal(|ui| {
                ui.label("Padding:");
                ui.add(egui::DragValue::new(&mut state.counter_padding).range(0..=10));
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn rule_builder_panel_compiles() {
        // Module compiles and is usable
    }
}
