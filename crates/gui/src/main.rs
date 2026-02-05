use eframe::egui;
use profile::{Config, Profile};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0]),
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

struct PhybkcApp {
    config: Option<Config>,
    current_profile: Option<Profile>,
    selected_view: View,
}

#[derive(PartialEq)]
enum View {
    Profiles,
    Scripts,
    Mappings,
}

impl PhybkcApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Try to load initial config
        let config = Config::load_from_file("sample/config.toml").ok();
        let mut app = Self {
            config,
            current_profile: None,
            selected_view: View::Profiles,
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
            View::Profiles => self.profiles_view(ui),
            View::Scripts => self.scripts_view(ui),
            View::Mappings => self.mappings_view(ui),
        });
    }
}

impl PhybkcApp {
    fn profiles_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Profile Management");
        ui.add_space(10.0);

        if let Some(config) = &self.config {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (name, path) in &config.profiles {
                    let is_active = self
                        .current_profile
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
                                                self.current_profile =
                                                    Profile::load_from_file(path).ok();
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

    fn scripts_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Active Scripts");
        ui.add_space(10.0);

        if let Some(profile) = &self.current_profile {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for script in &profile.scripts {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(script);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("Edit").clicked() {
                                        // Open in editor?
                                    }
                                },
                            );
                        });
                    });
                    ui.add_space(5.0);
                }
            });
        } else {
            ui.label("No profile selected.");
        }
    }

    fn mappings_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Key Mappings");
        ui.add_space(10.0);

        if let Some(profile) = &self.current_profile {
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
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();
    // In a real app, logic to load Inter or Roboto from file system could go here.
    // For now we use default proportional fonts.
    ctx.set_fonts(fonts);
}
