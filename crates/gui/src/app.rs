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
}

impl PhybkcApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load_from_file("sample/config.toml").ok();
        let mut app = Self {
            config,
            current_profile: None,
            selected_view: View::Profiles,
            editing_script: None,
        };
        app.load_default_profile();
        app
    }

    fn load_default_profile(&mut self) {
        if let Some(config) = &self.config {
            let path = config.profiles.get(&config.default_profile.default);
            if let Some(p) = path {
                self.current_profile = Profile::load_from_file(p).ok();
            }
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
            View::Profiles => {
                views::profiles::profiles_view(ui, &self.config, &mut self.current_profile)
            }
            View::Scripts => {
                views::scripts::scripts_view(ui, &self.current_profile, &mut self.editing_script)
            }
            View::Mappings => views::mappings::mappings_view(ui, &self.current_profile),
        });
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();
    ctx.set_fonts(fonts);
}
