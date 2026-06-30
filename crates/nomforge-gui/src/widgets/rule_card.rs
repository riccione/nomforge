#![allow(dead_code)]

use eframe::egui;

/// A collapsible card for grouping related rule inputs.
///
/// Usage:
/// ```ignore
/// rule_card(ui, "Find & Replace", |ui| {
///     ui.horizontal(|ui| {
///         ui.label("Find:");
///         ui.text_edit_singleline(&mut state.find);
///     });
///     ui.horizontal(|ui| {
///         ui.label("Replace:");
///         ui.text_edit_singleline(&mut state.replace);
///     });
/// });
/// ```
pub fn show(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::CollapsingHeader::new(egui::RichText::new(title).strong())
        .default_open(true)
        .show(ui, |ui| {
            add_contents(ui);
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn rule_card_compiles() {
        // Module compiles and is usable
    }
}
