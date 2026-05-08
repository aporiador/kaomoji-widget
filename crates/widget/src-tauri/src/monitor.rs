use std::{collections::HashMap, sync::Mutex};

use serde::Serialize;
use tauri::{LogicalPosition, State};

pub fn is_position_on_any_monitor(app: &tauri::App, pos: LogicalPosition<f64>) -> bool {
    match app.available_monitors() {
        Ok(monitors) => monitors.iter().any(|m| {
            let scale_factor = m.scale_factor();
            let m_pos = m.position().to_logical(scale_factor);
            let m_size: tauri::LogicalSize<f64> = m.size().to_logical(scale_factor);
            pos.x >= m_pos.x
                && pos.x < m_pos.x + m_size.width
                && pos.y >= m_pos.y
                && pos.y < m_pos.y + m_size.height
        }),
        Err(_) => false,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MonitorInfo {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale: f64,
    pub is_primary: bool,
    pub insets: Insets,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Insets {
    pub top: f64,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
}

#[derive(Default)]
pub struct MonitorCache(pub Mutex<HashMap<String, MonitorInfo>>);

#[tauri::command]
pub fn get_monitors(cache: State<'_, MonitorCache>) -> Result<Vec<MonitorInfo>, String> {
    Ok(cache.0.lock().unwrap().values().cloned().collect())
}

#[tauri::command]
pub fn get_notch_inset(name: String, cache: State<'_, MonitorCache>) -> Option<f64> {
    cache
        .0
        .lock()
        .unwrap()
        .get(&name)
        .map(|monitor| monitor.insets.top)
}

#[cfg(not(target_os = "macos"))]
pub fn collect_monitors(app: tauri::AppHandle) -> Result<MonitorCache, String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    let primary = app.primary_monitor().map_err(|e| e.to_string())?;
    let primary_name = primary.as_ref().and_then(|m| m.name());

    let mut infos = Vec::new();
    for (i, m) in monitors.iter().enumerate() {
        let scale = m.scale_factor();
        let pos = m.position().to_logical::<f64>(scale);
        let size = m.size().to_logical::<f64>(scale);
        let name = m
            .name()
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("Monitor {}", i + 1));
        let is_primary = primary_name == Some(&name);

        infos.push(MonitorInfo {
            name,
            scale,
            x: pos.x,
            y: pos.y,
            width: size.width,
            height: size.height,
            is_primary,
            insets: Insets {
                top: 0.0,
                left: 0.0,
                bottom: 0.0,
                right: 0.0,
            },
        });
    }

    let cache = MonitorCache(Mutex::new(
        infos
            .into_iter()
            .map(|info| (info.name.clone(), info))
            .collect::<HashMap<_, _>>(),
    ));
    Ok(cache)
}

#[cfg(target_os = "macos")]
pub fn collect_monitors(_app: tauri::AppHandle) -> Result<MonitorCache, String> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSScreen;

    let mtm =
        MainThreadMarker::new().ok_or_else(|| "must be called on the main thread".to_string())?;
    let screens = NSScreen::screens(mtm);

    // Primary screen = screens[0] in AppKit (the one with the menu bar / global origin).
    // Used to compute the Y-flip so other screens land in top-left coords.
    let primary_height = screens
        .iter()
        .next()
        .map(|s| s.frame().size.height)
        .unwrap_or(0.0);

    let infos: Vec<MonitorInfo> = screens
        .iter()
        .enumerate()
        .map(|(i, screen)| {
            let frame = screen.frame();
            let insets = screen.safeAreaInsets();
            let name = screen.localizedName().to_string();
            let name = if name.is_empty() {
                format!("Monitor {}", i + 1)
            } else {
                name
            };

            // AppKit Y is bottom-up from the primary screen's bottom-left.
            // Convert to top-left origin to match Tauri's position semantics.
            let y_top_left = primary_height - frame.origin.y - frame.size.height;

            MonitorInfo {
                name,
                x: frame.origin.x,
                y: y_top_left,
                width: frame.size.width,
                height: frame.size.height,
                scale: screen.backingScaleFactor(),
                is_primary: i == 0,
                insets: Insets {
                    top: insets.top,
                    left: insets.left,
                    bottom: insets.bottom,
                    right: insets.right,
                },
            }
        })
        .collect();

    let cache = MonitorCache(Mutex::new(
        infos
            .into_iter()
            .map(|info| (info.name.clone(), info))
            .collect::<HashMap<_, _>>(),
    ));
    Ok(cache)
}
