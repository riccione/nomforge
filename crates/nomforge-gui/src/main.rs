mod app;
mod state;

use app::NomforgeApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "nomforge",
        options,
        Box::new(|_cc| Ok(Box::new(NomforgeApp::default()))),
    )
}
