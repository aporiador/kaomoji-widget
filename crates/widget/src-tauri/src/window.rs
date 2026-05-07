use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

/// On macOS the app may not be the active application when a tray menu item
/// is clicked, so `makeKeyAndOrderFront:` is a no-op unless we activate first.
#[cfg(target_os = "macos")]
fn activate_app() {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    unsafe {
        let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![ns_app, activateIgnoringOtherApps: true];
    }
}

pub fn open_settings_window(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Bring the application to the foreground so the window can actually get focus.
    #[cfg(target_os = "macos")]
    activate_app();

    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let window =
        WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("/settings.html".into()))
            .title("Kaomoji Widget Settings")
            .inner_size(420.0, 520.0)
            .resizable(false)
            .decorations(true)
            .transparent(false)
            .always_on_top(false)
            .skip_taskbar(false)
            .center()
            .build()?;

    let _ = window.set_focus();

    // Clean up the window handle when closed so we can reopen it later
    let app_clone = app.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Destroyed = event {
            let _ = app_clone.get_webview_window("settings");
        }
    });

    Ok(())
}
