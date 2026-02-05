use eframe::egui;

pub fn scripts_view(ui: &mut egui::Ui, app: &mut crate::app::PhybkcApp) {
    ui.heading("Active Scripts");
    ui.horizontal(|ui| {
        ui.label("Add Script Path:");
        ui.text_edit_singleline(&mut app.new_script_path);
        if ui.button("✚ Add").clicked() {
            app.add_script_to_profile(&app.new_script_path.clone());
        }
    });

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    let mut close_editor = false;
    if let Some(script_data) = &mut app.editing_script {
        ui.horizontal(|ui| {
            if ui.button("⬅ Back").clicked() {
                close_editor = true;
            }
            ui.label(format!("Editing: {}", script_data.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Save").clicked() {
                    // Save to file
                    if std::fs::write(&script_data.0, &script_data.1).is_ok() {
                        println!("Saved script: {}", script_data.0);
                    }
                }
            });
        });
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(&mut script_data.1)
                    .font(egui::TextStyle::Monospace)
                    .code_editor(),
            );
        });

        if close_editor {
            app.editing_script = None;
        }
        return;
    }

    if let Some(profile) = &app.current_profile {
        let scripts = profile.scripts.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, script_path) in scripts.iter().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(script_path);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑").clicked() {
                                app.remove_script_from_profile(i);
                            }
                            if ui.button("Edit").clicked()
                                && let Ok(content) = std::fs::read_to_string(script_path)
                            {
                                app.editing_script = Some((script_path.clone(), content));
                            }
                        });
                    });
                });
                ui.add_space(5.0);
            }
        });
    } else {
        ui.label("No profile selected.");
    }
}
