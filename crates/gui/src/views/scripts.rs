use eframe::egui;
use profile::Profile;

pub fn scripts_view(
    ui: &mut egui::Ui,
    current_profile: &Option<Profile>,
    editing_script: &mut Option<(String, String)>,
) {
    ui.heading("Active Scripts");
    ui.add_space(10.0);

    let mut close_editor = false;
    if let Some(script_data) = editing_script {
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
            *editing_script = None;
        }
        return;
    }

    if let Some(profile) = current_profile {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for script_path in &profile.scripts {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(script_path);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Edit").clicked() {
                                if let Ok(content) = std::fs::read_to_string(script_path) {
                                    *editing_script = Some((script_path.clone(), content));
                                }
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
