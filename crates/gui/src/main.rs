mod app;
mod views;

use app::PhybkcApp;
use eframe::egui;

fn main() -> eframe::Result {
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
        Box::new(|cc| {
            // Use dark mode by default for premium feel
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(PhybkcApp::new(cc)))
        }),
    )
}

fn load_icon() -> egui::IconData {
    let (width, height) = (32, 32);
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let r = (x * 4) as u8;
            let g = 100;
            let b = (y * 4) as u8;
            let a = 255;
            rgba.push(r);
            rgba.push(g);
            rgba.push(b);
            rgba.push(a);
        }
    }
    egui::IconData {
        rgba,
        width,
        height,
    }
}
