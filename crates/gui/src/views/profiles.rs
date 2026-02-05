use eframe::egui;
use profile::{Config, Profile};

pub fn profiles_view(
    ui: &mut egui::Ui,
    config: &Option<Config>,
    current_profile: &mut Option<Profile>,
) {
    ui.heading("Profile Management");
    ui.add_space(10.0);

    if let Some(config) = config {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (name, path) in &config.profiles {
                let is_active = current_profile
                    .as_ref()
                    .map(|p| &p.name == name)
                    .unwrap_or(false);

                egui::Frame::group(ui.style())
                    .fill(if is_active {
                        egui::Color32::from_rgb(40, 50, 80)
                    } else {
                        egui::Color32::from_gray(30)
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(name).strong().size(18.0));
                                ui.label(egui::RichText::new(path).weak());
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if !is_active {
                                        if ui.button("Activate").clicked() {
                                            *current_profile = Profile::load_from_file(path).ok();
                                        }
                                    } else {
                                        ui.label("Active");
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(10.0);
            }
        });
    } else {
        ui.label("Config not loaded.");
    }
}
