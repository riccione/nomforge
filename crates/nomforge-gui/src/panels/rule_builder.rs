use eframe::egui;

use crate::state::State;

/// What the rule panel observed this frame.
#[derive(Debug, Default, Clone, Copy)]
pub struct RuleBuilderOutput {
    /// A field was submitted with Enter.
    pub submitted: bool,
    /// A rule value changed.
    pub changed: bool,
}

/// Render the rule builder panel (simple mode by default, regex in advanced mode).
///
/// The output tells the caller whether to refresh the preview (Enter) or to
/// schedule a live preview (typing).
pub fn show(ui: &mut egui::Ui, state: &mut State) -> RuleBuilderOutput {
    let mut output = RuleBuilderOutput::default();

    ui.collapsing("Rules", |ui| {
        ui.horizontal(|ui| {
            ui.label("Find:");
            let response = ui.text_edit_singleline(&mut state.find);
            output.submitted |= crate::panels::submitted(ui, &response);
            output.changed |= response.changed();
        });
        ui.horizontal(|ui| {
            ui.label("Replace:");
            let response = ui.text_edit_singleline(&mut state.replace);
            output.submitted |= crate::panels::submitted(ui, &response);
            output.changed |= response.changed();
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Prefix:");
            let response = ui.text_edit_singleline(&mut state.prefix);
            output.submitted |= crate::panels::submitted(ui, &response);
            output.changed |= response.changed();
        });
        ui.horizontal(|ui| {
            ui.label("Suffix:");
            let response = ui.text_edit_singleline(&mut state.suffix);
            output.submitted |= crate::panels::submitted(ui, &response);
            output.changed |= response.changed();
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Remove:");
            let response = ui.text_edit_singleline(&mut state.remove);
            output.submitted |= crate::panels::submitted(ui, &response);
            output.changed |= response.changed();
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Case:");
            let mut changed = false;
            egui::ComboBox::from_id_salt("case_combo")
                .selected_text(&state.case)
                .show_ui(ui, |ui| {
                    changed |= ui
                        .selectable_value(&mut state.case, String::new(), "(none)")
                        .changed();
                    changed |= ui
                        .selectable_value(&mut state.case, "upper".into(), "Upper")
                        .changed();
                    changed |= ui
                        .selectable_value(&mut state.case, "lower".into(), "Lower")
                        .changed();
                    changed |= ui
                        .selectable_value(&mut state.case, "title".into(), "Title")
                        .changed();
                });
            output.changed |= changed;
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Change Extension (without dot):");
            let response = ui.text_edit_singleline(&mut state.ext_change);
            output.submitted |= crate::panels::submitted(ui, &response);
            output.changed |= response.changed();
        });

        // Advanced mode: regex and counter fields
        ui.separator();
        ui.checkbox(&mut state.advanced_mode, "Advanced mode");
        if state.advanced_mode {
            ui.horizontal(|ui| {
                ui.label("Regex:");
                let response = ui.text_edit_singleline(&mut state.regex);
                output.submitted |= crate::panels::submitted(ui, &response);
                output.changed |= response.changed();
            });
            ui.horizontal(|ui| {
                ui.label("Replacement:");
                let response = ui.text_edit_singleline(&mut state.replacement);
                output.submitted |= crate::panels::submitted(ui, &response);
                output.changed |= response.changed();
            });

            ui.separator();

            ui.label("Counter:");
            ui.horizontal(|ui| {
                ui.label("Start:");
                let response =
                    ui.add(egui::DragValue::new(&mut state.counter_start).range(0..=10000));
                output.submitted |= crate::panels::submitted(ui, &response);
                output.changed |= response.changed();
            });
            ui.horizontal(|ui| {
                ui.label("Padding:");
                let response =
                    ui.add(egui::DragValue::new(&mut state.counter_padding).range(0..=10));
                output.submitted |= crate::panels::submitted(ui, &response);
                output.changed |= response.changed();
            });
            ui.horizontal(|ui| {
                ui.label("Position:");
                let mut changed = false;
                egui::ComboBox::from_id_salt("counter_position")
                    .selected_text(&state.counter_position)
                    .show_ui(ui, |ui| {
                        changed |= ui
                            .selectable_value(
                                &mut state.counter_position,
                                "prefix".into(),
                                "Prefix",
                            )
                            .changed();
                        changed |= ui
                            .selectable_value(
                                &mut state.counter_position,
                                "suffix".into(),
                                "Suffix",
                            )
                            .changed();
                        changed |= ui
                            .selectable_value(
                                &mut state.counter_position,
                                "replace".into(),
                                "Replace stem",
                            )
                            .changed();
                    });
                output.changed |= changed;
            });
        }
    });

    output
}

#[cfg(test)]
mod tests {
    #[test]
    fn rule_builder_panel_compiles() {
        // Module compiles and is usable
    }
}
