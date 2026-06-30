mod app;
mod panels;
mod state;

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
        "nomforge",
        options,
        Box::new(|_cc| Ok(Box::new(NomforgeApp::default()))),
    )
}
