mod app;
mod views;

use app::PhybkcApp;
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Phybkc Dashboard",
        options,
        Box::new(|cc| {
            // Use dark mode by default for premium feel
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(PhybkcApp::new(cc)))
        }),
    )
}
