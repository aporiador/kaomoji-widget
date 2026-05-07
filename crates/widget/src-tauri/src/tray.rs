use crate::window::open_settings_window;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, PhysicalPosition};
use tauri_plugin_store::StoreExt;

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let tray_menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "show", "Show", true, None::<&str>)?,
            &MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "reset", "Reset Position", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
        ],
    )?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or("no default window icon available")?;

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&tray_menu)
        .on_menu_event(|app, event| {
            let window = app.get_webview_window("main");
            match event.id.as_ref() {
                "show" => {
                    if let Some(w) = window {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                "hide" => {
                    if let Some(w) = window {
                        let _ = w.hide();
                    }
                }
                "settings" => {
                    if let Err(e) = open_settings_window(app) {
                        eprintln!("Failed to open settings window: {e}");
                    }
                }
                "reset" => {
                    if let Ok(store) = app.store("settings.bin") {
                        let _ = store.delete("window_position");
                        let _ = store.save();
                    }
                    if let Some(w) = window {
                        let default = PhysicalPosition::new(100, 100);
                        let _ = w.set_position(tauri::Position::Physical(default));
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
