use eframe::egui;
use profile::Profile;

pub fn mappings_view(ui: &mut egui::Ui, current_profile: &Option<Profile>) {
    ui.heading("Key Mappings");
    ui.add_space(10.0);

    if let Some(profile) = current_profile {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("mapping_grid")
                .num_columns(2)
                .spacing([40.0, 10.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Physical Key");
                    ui.label("Mapped To");
                    ui.end_row();

                    for (sc, name) in &profile.keys {
                        ui.label(sc);
                        ui.label(name);
                        ui.end_row();
                    }
                });
        });
    } else {
        ui.label("No profile selected.");
    }
}
