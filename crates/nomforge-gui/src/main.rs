mod app;
mod panels;
mod state;
mod widgets;

use app::NomforgeApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_position([100.0, 100.0])
            .with_maximized(false),
        ..Default::default()
    };

    eframe::run_native(
        concat!("nomforge v", env!("CARGO_PKG_VERSION")),
        options,
        Box::new(|cc| {
            // egui defaults render too small; bump every text style by 2pt
            // (both dark and light themes) while keeping font families.
            cc.egui_ctx.all_styles_mut(|style| {
                for font_id in style.text_styles.values_mut() {
                    font_id.size += 2.0;
                }
            });
            Ok(Box::new(NomforgeApp::default()))
        }),
    )
}
