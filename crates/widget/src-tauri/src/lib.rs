mod commands;
mod ipc;
mod monitor;
mod settings;
mod tray;
mod window;

use crate::ipc::run_ipc_listener;
use crate::monitor::is_position_on_any_monitor;
use crate::settings::{get_settings, set_settings};
use crate::tray::setup_tray;
use serde_json::json;
use std::time::Duration;
use tauri::{Manager, PhysicalPosition, WindowEvent};
use tauri_plugin_store::StoreExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![get_settings, set_settings])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let handle = app.handle().clone();
            tauri::async_runtime::spawn_blocking(move || {
                run_ipc_listener(handle);
            });

            // ── Position persistence ──────────────────────────────────
            if let Some(window) = app.get_webview_window("main") {
                let store = app.store("settings.bin")?;

                // Restore saved position if it lands on a connected monitor
                if let Some(pos_value) = store.get("window_position") {
                    if let (Some(x), Some(y)) = (
                        pos_value.get("x").and_then(|v| v.as_f64()),
                        pos_value.get("y").and_then(|v| v.as_f64()),
                    ) {
                        let pos = tauri::LogicalPosition::new(x, y);
                        if is_position_on_any_monitor(app, pos) {
                            let _ = window.set_position(tauri::Position::Logical(pos));
                        }
                    }
                }

                // Debounced save on move (500 ms)
                let (pos_tx, mut pos_rx) = tokio::sync::watch::channel(PhysicalPosition::new(0, 0));
                let app_handle_for_save = app.handle().clone();
                let store_for_save = store.clone();
                tauri::async_runtime::spawn(async move {
                    loop {
                        if pos_rx.changed().await.is_err() {
                            break;
                        }
                        let pos = *pos_rx.borrow();
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        if pos_rx.has_changed().unwrap_or(true) {
                            continue;
                        }
                        if let Some(window) = app_handle_for_save.get_webview_window("main") {
                            let scale_factor = window.scale_factor().unwrap_or(1.0);
                            let logical_pos: tauri::LogicalPosition<f64> = pos.to_logical(scale_factor);
                            let _ = store_for_save.set(
                                "window_position",
                                json!({"x": logical_pos.x, "y": logical_pos.y}),
                            );
                            let _ = store_for_save.save();
                        }
                    }
                });

                window.on_window_event(move |event| {
                    if let WindowEvent::Moved(pos) = event {
                        let _ = pos_tx.send(*pos);
                    }
                });
            }

            // ── System tray ───────────────────────────────────────────
            if let Err(e) = setup_tray(app) {
                eprintln!("Warning: failed to setup system tray: {e}");
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
