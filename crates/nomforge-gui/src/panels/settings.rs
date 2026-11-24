use eframe::egui;

use crate::settings::{self, MAX_FONT_SIZE, MIN_FONT_SIZE, ThemeSetting};
use crate::state::{State, StatusKind};

/// Render the settings modal (if open).
///
/// Changes apply live without restarting the app. Theme and the auto-scan
/// toggle are discrete and apply immediately; the font size is committed on
/// slider release or Enter in the input.
pub fn show(ui: &mut egui::Ui, state: &mut State) {
    if !state.show_settings_modal {
        return;
    }

    let ctx = ui.ctx().clone();
    let modal = egui::Modal::new(egui::Id::unique("settings")).show(&ctx, |ui| {
        ui.set_width(320.0);
        ui.heading("Settings");
        ui.add_space(8.0);

        // Theme: a radio click is a discrete action, so apply it right away.
        ui.label("Theme");
        ui.horizontal(|ui| {
            ui.radio_value(&mut state.draft_theme, ThemeSetting::Light, "Light");
            ui.radio_value(&mut state.draft_theme, ThemeSetting::Dark, "Dark");
            ui.radio_value(&mut state.draft_theme, ThemeSetting::System, "System");
        });
        if state.draft_theme != state.settings.theme {
            state.settings.theme = state.draft_theme;
            ctx.set_theme(egui::ThemePreference::from(state.settings.theme));
            persist(state);
        }

        ui.add_space(8.0);

        // Font size: only commit on slider release or Enter in the input.
        ui.label("Font size (px)");
        let slider = ui.add(
            egui::Slider::new(&mut state.draft_font_size, MIN_FONT_SIZE..=MAX_FONT_SIZE)
                .step_by(1.0)
                .show_value(false),
        );
        let input = ui
            .horizontal(|ui| {
                ui.label("Size:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.draft_font_size_input)
                        .desired_width(60.0),
                )
            })
            .inner;

        // Keep the input in sync while the user drags the slider (but never
        // overwrite what they are currently typing).
        if slider.changed() && !input.has_focus() {
            state.draft_font_size_input = format!("{:.0}", state.draft_font_size);
        }

        if slider.drag_stopped() {
            commit_font_size(&ctx, state, state.draft_font_size);
        } else if input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            match state.draft_font_size_input.trim().parse::<f32>() {
                Ok(size) => {
                    let size = size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
                    state.draft_font_size = size;
                    state.draft_font_size_input = format!("{size:.0}");
                    commit_font_size(&ctx, state, size);
                }
                Err(_) => {
                    state.draft_font_size_input = format!("{:.0}", state.settings.font_size);
                }
            }
        }

        ui.add_space(8.0);
        let mut auto_scan = state.settings.auto_scan;
        if ui
            .checkbox(
                &mut auto_scan,
                "Scan automatically when a directory is selected",
            )
            .changed()
        {
            state.settings.auto_scan = auto_scan;
            persist(state);
        }

        let mut live_preview = state.settings.live_preview;
        if ui
            .checkbox(&mut live_preview, "Preview while typing rules and filters")
            .changed()
        {
            state.settings.live_preview = live_preview;
            persist(state);
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);
        if ui.button("Close").clicked() {
            state.show_settings_modal = false;
        }
    });

    if modal.should_close() {
        state.show_settings_modal = false;
    }
}

/// Apply a committed font size and persist it.
fn commit_font_size(ctx: &egui::Context, state: &mut State, size: f32) {
    state.settings.font_size = size;
    ctx.set_zoom_factor(state.settings.zoom_factor());
    persist(state);
}

/// Persist the current settings, reporting failures in the status bar.
fn persist(state: &mut State) {
    if let Err(e) = settings::save(&state.settings) {
        state.set_status(StatusKind::Error, format!("Settings save error: {e}"));
    }
}
