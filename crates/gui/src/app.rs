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
    pub new_script_path: String,
    pub last_scancode: Option<u16>,
    pub previous_vk_states: [bool; 256],
}

impl PhybkcApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = if let Ok(cfg) = Config::load_from_file("config.toml") {
            Some(cfg)
        } else {
            let default_cfg = Config::default();
            let _ = default_cfg.save_to_file("config.toml");
            Some(default_cfg)
        };
        let mut app = Self {
            config,
            current_profile: None,
            selected_view: View::Profiles,
            editing_script: None,
            new_profile_name: String::new(),
            import_path: String::new(),
            new_script_path: String::new(),
            last_scancode: None,
            previous_vk_states: [false; 256],
        };
        app.load_default_profile();
        app
    }

    fn load_default_profile(&mut self) {
        if let Some(config) = &self.config
            && let Some(profile_path) = config.profiles.get(&config.default_profile.default)
        {
            self.current_profile = Profile::load_from_file(profile_path).ok();
        }
    }

    pub fn save_config(&self) {
        if let Some(config) = &self.config
            && let Err(e) = config.save_to_file("config.toml")
        {
            eprintln!("Failed to save config.toml: {:?}", e);
        }
    }

    pub fn create_profile(&mut self, name: &str) {
        let name = name.trim();
        if name.is_empty() {
            return;
        }
        let path = format!("profiles/{}.json", name);
        let new_profile = Profile {
            name: name.to_string(),
            keyboard: "Default".to_string(),
            scripts: vec![],
            keys: std::collections::HashMap::new(),
        };
        if let Some(config) = &mut self.config {
            let _ = std::fs::create_dir_all("profiles");
            if let Err(e) = new_profile.save_to_file(&path) {
                eprintln!("Failed to save new profile to {}: {:?}", path, e);
            } else {
                config.profiles.insert(name.to_string(), path);
                self.save_config();
            }
        } else {
            eprintln!("Config not loaded - cannot create profile {}", name);
        }
    }

    pub fn delete_profile(&mut self, name: &str) {
        if let Some(config) = &mut self.config
            && let Some(path) = config.profiles.remove(name)
        {
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

    pub fn import_profile(&mut self, path: &str) {
        let path = path.trim().trim_matches('"');
        if path.is_empty() {
            return;
        }
        if let Ok(profile) = Profile::load_from_file(path) {
            let name = profile.name.clone();
            let new_path = format!("profiles/{}.json", name);
            let _ = std::fs::create_dir_all("profiles");

            // Avoid copying if the file is already at the target location
            let should_copy = if let Ok(canon_src) = std::fs::canonicalize(path)
                && let Ok(canon_dst) = std::fs::canonicalize(&new_path)
            {
                canon_src != canon_dst
            } else {
                true
            };

            let copy_ok = if should_copy {
                if let Err(e) = std::fs::copy(path, &new_path) {
                    eprintln!(
                        "Failed to copy profile from {} to {}: {:?}",
                        path, new_path, e
                    );
                    false
                } else {
                    true
                }
            } else {
                true
            };

            if copy_ok && let Some(config) = &mut self.config {
                config.profiles.insert(name, new_path);
                self.save_config();
            } else if copy_ok && self.config.is_none() {
                eprintln!(
                    "Config not loaded - imported file to {}, but couldn't update config.toml",
                    new_path
                );
            }
        } else {
            eprintln!("Failed to load profile from {}", path);
        }
    }

    pub fn export_profile(&self, name: &str, target_dir: &str) {
        if let (Some(_config), Some(path)) = (
            &self.config,
            self.config.as_ref().and_then(|c| c.profiles.get(name)),
        ) && let Ok(profile) = Profile::load_from_file(path)
        {
            let _ = std::fs::create_dir_all(target_dir);
            let profiles_dir = std::path::Path::new(target_dir).join("profiles");
            let scripts_dir = std::path::Path::new(target_dir).join("scripts");
            let _ = std::fs::create_dir_all(&profiles_dir);
            let _ = std::fs::create_dir_all(&scripts_dir);

            let target_json = profiles_dir.join(format!("{}.json", name));
            let _ = std::fs::copy(path, &target_json);

            for script in &profile.scripts {
                let script_path = std::path::Path::new(script);
                if let Some(file_name) = script_path.file_name() {
                    let target_script = scripts_dir.join(file_name);
                    let _ = std::fs::copy(script, target_script);
                }
            }
        }
    }

    pub fn add_script_to_profile(&mut self, script_path: &str) {
        if script_path.is_empty() {
            return;
        }
        if let (Some(profile), Some(config)) = (&mut self.current_profile, &self.config) {
            profile.scripts.push(script_path.to_string());
            if let Some(path) = config.profiles.get(&profile.name) {
                let _ = profile.save_to_file(path);
            }
        }
        self.new_script_path.clear();
    }

    pub fn remove_script_from_profile(&mut self, index: usize) {
        if let (Some(profile), Some(config)) = (&mut self.current_profile, &self.config)
            && index < profile.scripts.len()
        {
            profile.scripts.remove(index);
            if let Some(path) = config.profiles.get(&profile.name) {
                let _ = profile.save_to_file(path);
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
            View::Mappings => {
                // Capture ScanCode using state transitions for robustness
                for vk in 1..256u32 {
                    // Skip generic/ambiguous VKs (VK_SHIFT, VK_CONTROL, VK_MENU)
                    if vk == 0x10 || vk == 0x11 || vk == 0x12 {
                        continue;
                    }

                    let is_down = unsafe {
                        (windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(
                            vk as i32,
                        ) as u32
                            & 0x8000)
                            != 0
                    };

                    let prev_down = self.previous_vk_states[vk as usize];
                    if is_down && !prev_down {
                        // New key press detected
                        let sc = unsafe {
                            windows_sys::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyW(vk, 0)
                        };
                        if sc != 0 {
                            self.last_scancode = Some(sc as u16);
                        }
                    }
                    self.previous_vk_states[vk as usize] = is_down;
                }
                views::mappings::mappings_view(ui, self)
            }
        });
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();
    ctx.set_fonts(fonts);
}
