use profile::Config;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{IsMenuItem, Menu, MenuEvent, MenuItem, Submenu},
};

#[derive(Debug)]
pub enum TrayAction {
    None,
    Quit,
    Reload,
    SwitchProfile(String),
}

pub fn init_tray(config: &Config) -> anyhow::Result<TrayIcon> {
    let tray_menu = Menu::new();
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);
    let reload_item = MenuItem::with_id("reload", "Reload Profile", true, None);

    let profile_submenu = Submenu::new("Profiles", true);
    for name in config.profiles.keys() {
        let item = MenuItem::with_id(format!("profile:{}", name), name, true, None);
        profile_submenu.append(&item)?;
    }

    tray_menu.append_items(&[
        &profile_submenu as &dyn IsMenuItem,
        &reload_item as &dyn IsMenuItem,
        &quit_item as &dyn IsMenuItem,
    ])?;

    let tooltip = format!("phybkc - {}", config.default_profile.default);
    let icon = load_icon();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip(tooltip)
        .with_icon(icon)
        .build()?;

    Ok(tray_icon)
}

pub fn handle_tray_events() -> TrayAction {
    if let Ok(event) = MenuEvent::receiver().try_recv() {
        let id = event.id.0.as_str();
        if id == "quit" {
            return TrayAction::Quit;
        } else if id == "reload" {
            return TrayAction::Reload;
        } else if id.starts_with("profile:") {
            let profile_name = id.trim_start_matches("profile:").to_string();
            return TrayAction::SwitchProfile(profile_name);
        }
    }
    TrayAction::None
}

fn load_icon() -> tray_icon::Icon {
    let (width, height) = (32, 32);
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            // Create a simple blue/purple gradient
            let r = (x * 4) as u8;
            let g = 100;
            let b = (y * 4) as u8;
            let a = 255;
            rgba.push(r);
            rgba.push(g);
            rgba.push(b);
            rgba.push(a);
        }
    }
    tray_icon::Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}
