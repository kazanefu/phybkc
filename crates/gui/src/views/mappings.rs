use eframe::egui;

pub fn mappings_view(ui: &mut egui::Ui, app: &mut crate::app::PhybkcApp) {
    ui.heading("Key Mappings");
    ui.add_space(10.0);

    // ScanCode Detector Section
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("󰌌 ScanCode Detector:");
            if let Some(sc) = app.last_scancode {
                let name = profile::get_name(sc).unwrap_or("Unknown");
                ui.colored_label(
                    egui::Color32::from_rgb(0, 255, 127),
                    format!("0x{:02X} ({})", sc, name),
                );
            } else {
                ui.label("Press any key...");
            }
            if ui.button("Clear").clicked() {
                app.last_scancode = None;
            }
        });
    });

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    let mut profile = if let Some(p) = &app.current_profile {
        p.clone()
    } else {
        ui.label("No profile selected.");
        return;
    };

    let mut changed = false;

    ui.horizontal(|ui| {
        if ui.button("✚ Add Mapping").clicked() {
            profile.keys.insert("0x00".to_string(), "None".to_string());
            changed = true;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Save Changes").clicked()
                && let Some(config) = &app.config
                && let Some(path) = config.profiles.get(&profile.name)
            {
                let _ = profile.save_to_file(path);
                app.current_profile = Some(profile.clone());
            }
        });
    });

    ui.add_space(10.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("mapping_grid")
            .num_columns(3)
            .spacing([20.0, 10.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Physical Key (Hex)");
                ui.label("Mapped To (Virtual Name)");
                ui.label("Action");
                ui.end_row();

                let mut keys_to_remove = Vec::new();
                let mut keys_to_update = Vec::new();

                for (sc, name) in &profile.keys {
                    let mut sc_edit = sc.clone();
                    let mut name_edit = name.clone();

                    ui.text_edit_singleline(&mut sc_edit);
                    ui.text_edit_singleline(&mut name_edit);

                    if ui.button("🗑").clicked() {
                        keys_to_remove.push(sc.clone());
                    }
                    ui.end_row();

                    if sc_edit != *sc || name_edit != *name {
                        keys_to_update.push((sc.clone(), sc_edit, name_edit));
                    }
                }

                for sc in keys_to_remove {
                    profile.keys.remove(&sc);
                    changed = true;
                }

                for (old_sc, new_sc, new_name) in keys_to_update {
                    profile.keys.remove(&old_sc);
                    profile.keys.insert(new_sc, new_name);
                    changed = true;
                }
            });
    });

    if changed {
        app.current_profile = Some(profile);
    }
}
