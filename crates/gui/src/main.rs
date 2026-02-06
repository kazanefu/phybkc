mod app;
mod hook;
mod views;

use app::PhybkcApp;
use eframe::egui;

fn main() -> eframe::Result {
    // Start the keyboard hook thread to capture physical scancodes
    let scancode_slot = hook::start_hook_thread();

    let icon = load_icon();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Phybkc Dashboard",
        options,
        Box::new(move |cc| {
            // Use dark mode by default for premium feel
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(PhybkcApp::new(cc, scancode_slot)))
        }),
    )
}

fn load_icon() -> egui::IconData {
    let icon_bytes = include_bytes!("../../../assets/icon.png");
    let image = image::load_from_memory(icon_bytes).expect("Failed to load icon");
    let image = image.to_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}
