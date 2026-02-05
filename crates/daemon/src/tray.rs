use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};

pub fn init_tray() -> anyhow::Result<TrayIcon> {
    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("Quit", true, None);
    let reload_item = MenuItem::new("Reload Profile", true, None);

    tray_menu.append_items(&[
        &reload_item as &dyn tray_icon::menu::IsMenuItem,
        &quit_item as &dyn tray_icon::menu::IsMenuItem,
    ])?;

    // For now, use a blank or simple icon.
    // In a real app, we'd load an ico file.
    let icon = load_icon();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("phybkc daemon")
        .with_icon(icon)
        .build()?;

    Ok(tray_icon)
}

pub fn handle_tray_events() {
    if let Ok(event) = MenuEvent::receiver().try_recv() {
        println!("Tray event: {:?}", event);
        // Handle events here (Quit, Reload, etc.)
        // For Quit, we might want to signal the main loop to exit.
    }
}

fn load_icon() -> tray_icon::Icon {
    let (width, height) = (32, 32);
    let rgba = vec![0u8; (width * height * 4) as usize];
    tray_icon::Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}
