use eframe::egui;
use profile::Profile;

pub fn profiles_view(ui: &mut egui::Ui, app: &mut crate::app::PhybkcApp) {
    ui.heading("Profile Management");
    ui.add_space(10.0);

    // --- Create / Import Section ---
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("New Profile:");
            ui.text_edit_singleline(&mut app.new_profile_name);
            if ui.button("Create").clicked() {
                app.create_profile(&app.new_profile_name.clone());
                app.new_profile_name.clear();
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Import path:");
            ui.text_edit_singleline(&mut app.import_path);
            if ui.button("Import").clicked() {
                app.import_profile(&app.import_path.clone());
                app.import_path.clear();
            }
        });
    });

    ui.add_space(20.0);

    if let Some(config) = app.config.clone() {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut profile_names: Vec<_> = config.profiles.keys().cloned().collect();
            profile_names.sort();

            for name in profile_names {
                let path = config.profiles.get(&name).unwrap().clone();
                let is_active = app
                    .current_profile
                    .as_ref()
                    .map(|p| p.name == name)
                    .unwrap_or(false);
                let is_default = config.default_profile.default == name;

                egui::Frame::group(ui.style())
                    .fill(if is_active {
                        egui::Color32::from_rgb(40, 50, 80)
                    } else {
                        egui::Color32::from_gray(30)
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&name).strong().size(18.0).color(egui::Color32::WHITE));
                                    if is_default {
                                        ui.label(egui::RichText::new("(Default)").small().weak().color(egui::Color32::WHITE));
                                    }
                                });
                                ui.label(egui::RichText::new(&path).weak().color(egui::Color32::WHITE));
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("🗑").on_hover_text("Delete").clicked() {
                                        app.delete_profile(&name);
                                    }

                                    if ui.button("📤").on_hover_text("Export").clicked() {
                                        let export_dir = format!("exports/{}", name);
                                        app.export_profile(&name, &export_dir);
                                    }

                                    if !is_default
                                        && ui.button("Set Default").clicked() {
                                            app.set_default_profile(&name);
                                        }

                                    if !is_active {
                                        if ui.button("Activate").clicked() {
                                            app.current_profile =
                                                Profile::load_from_file(&path).ok();
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
