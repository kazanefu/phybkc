use crate::views;
use eframe::egui;
use profile::{Config, Profile};

#[derive(PartialEq)]
pub enum View {
    Profiles,
    Scripts,
    Mappings,
}

pub struct PhybkcApp {
    pub config: Option<Config>,
    pub current_profile: Option<Profile>,
    pub selected_view: View,
    pub editing_script: Option<(String, String)>, // (Path, Content)

    // Profile Management State
    pub new_profile_name: String,
    pub import_path: String,
}

impl PhybkcApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load_from_file("sample/config.toml").ok();
        let mut app = Self {
            config,
            current_profile: None,
            selected_view: View::Profiles,
            editing_script: None,
            new_profile_name: String::new(),
            import_path: String::new(),
        };
        app.load_default_profile();
        app
    }

    fn load_default_profile(&mut self) {
        if let Some(config) = &self.config {
            if let Some(profile_path) = config.profiles.get(&config.default_profile.default) {
                self.current_profile = Profile::load_from_file(profile_path).ok();
            }
        }
    }

    pub fn save_config(&self) {
        if let Some(config) = &self.config {
            let _ = config.save_to_file("sample/config.toml");
        }
    }

    pub fn create_profile(&mut self, name: &str) {
        if name.is_empty() {
            return;
        }
        let path = format!("sample/{}.json", name);
        let new_profile = Profile {
            name: name.to_string(),
            keyboard: "Default".to_string(),
            scripts: vec![],
            keys: std::collections::HashMap::new(),
        };
        if let Some(config) = &mut self.config {
            if new_profile.save_to_file(&path).is_ok() {
                config.profiles.insert(name.to_string(), path);
                self.save_config();
            }
        }
    }

    pub fn delete_profile(&mut self, name: &str) {
        if let Some(config) = &mut self.config {
            if let Some(path) = config.profiles.remove(name) {
                let _ = std::fs::remove_file(path);
                if config.default_profile.default == name {
                    config.default_profile.default =
                        config.profiles.keys().next().cloned().unwrap_or_default();
                }
                self.save_config();
                if self
                    .current_profile
                    .as_ref()
                    .map(|p| p.name == name)
                    .unwrap_or(false)
                {
                    self.current_profile = None;
                    self.load_default_profile();
                }
            }
        }
    }

    pub fn import_profile(&mut self, path: &str) {
        if let Ok(profile) = Profile::load_from_file(path) {
            let name = profile.name.clone();
            let new_path = format!("sample/{}.json", name);
            if std::fs::copy(path, &new_path).is_ok() {
                if let Some(config) = &mut self.config {
                    config.profiles.insert(name, new_path);
                    self.save_config();
                }
            }
        }
    }

    pub fn export_profile(&self, name: &str, target_dir: &str) {
        if let Some(config) = &self.config {
            if let Some(path) = config.profiles.get(name) {
                if let Ok(profile) = Profile::load_from_file(path) {
                    let _ = std::fs::create_dir_all(target_dir);
                    let target_json =
                        std::path::Path::new(target_dir).join(format!("{}.json", name));
                    let _ = std::fs::copy(path, &target_json);

                    for script in &profile.scripts {
                        let script_path = std::path::Path::new(script);
                        if let Some(file_name) = script_path.file_name() {
                            let target_script = std::path::Path::new(target_dir).join(file_name);
                            let _ = std::fs::copy(script, target_script);
                        }
                    }
                }
            }
        }
    }

    pub fn set_default_profile(&mut self, name: &str) {
        if let Some(config) = &mut self.config {
            config.default_profile.default = name.to_string();
            self.save_config();
        }
    }
}

impl eframe::App for PhybkcApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        setup_custom_fonts(ctx);

        egui::SidePanel::left("side_panel")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.heading("Phybkc");
                    ui.add_space(20.0);
                });

                ui.separator();
                ui.add_space(10.0);

                ui.selectable_value(&mut self.selected_view, View::Profiles, " 󰀵  Profiles");
                ui.add_space(5.0);
                ui.selectable_value(&mut self.selected_view, View::Scripts, " 󰎆  Scripts");
                ui.add_space(5.0);
                ui.selectable_value(&mut self.selected_view, View::Mappings, " 󰌌  Mappings");

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                    ui.add_space(10.0);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.selected_view {
            View::Profiles => views::profiles::profiles_view(ui, self),
            View::Scripts => views::scripts::scripts_view(ui, self),
            View::Mappings => views::mappings::mappings_view(ui, self),
        });
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();
    ctx.set_fonts(fonts);
}
