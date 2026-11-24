//! nomforge GUI: an egui/eframe frontend for the core rename engine.

mod app;
mod job;
mod panels;
mod settings;
mod state;
mod widgets;

use app::NomforgeApp;
use state::State;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_position([100.0, 100.0])
            .with_maximized(false),
        ..Default::default()
    };

    let settings = settings::load();

    eframe::run_native(
        concat!("nomforge v", env!("CARGO_PKG_VERSION")),
        options,
        Box::new(move |cc| {
            // egui defaults render too small; bump every text style by 2pt
            // (both dark and light themes) while keeping font families.
            cc.egui_ctx.all_styles_mut(|style| {
                for font_id in style.text_styles.values_mut() {
                    font_id.size += 2.0;
                }
            });
            // Apply the persisted theme and font size before the first frame.
            cc.egui_ctx
                .set_theme(eframe::egui::ThemePreference::from(settings.theme));
            cc.egui_ctx.set_zoom_factor(settings.zoom_factor());
            Ok(Box::new(NomforgeApp::new(State {
                settings,
                ..Default::default()
            })))
        }),
    )
}
