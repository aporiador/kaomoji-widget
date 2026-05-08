use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;

use crate::monitor::MonitorCache;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_font_color")]
    pub font_color: String,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default = "default_text_shadow")]
    pub text_shadow: bool,
    #[serde(default = "default_text_shadow_color")]
    pub text_shadow_color: String,
    #[serde(default = "default_text_shadow_opacity")]
    pub text_shadow_opacity: f64,
    #[serde(default = "default_background_color")]
    pub background_color: String,
    #[serde(default = "default_background_opacity")]
    pub background_opacity: f64,
    #[serde(default = "default_notch_mode")]
    pub notch_mode: bool,
    #[serde(default)]
    pub notch_monitor: Option<String>,
}

fn default_theme() -> String {
    "system".to_string()
}
fn default_font_color() -> String {
    "#d97757".to_string()
}
fn default_font_family() -> String {
    "-apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, Helvetica, Arial, sans-serif"
        .to_string()
}
fn default_font_size() -> f64 {
    90.0
}
fn default_opacity() -> f64 {
    1.0
}
fn default_text_shadow() -> bool {
    true
}
fn default_text_shadow_color() -> String {
    "#000000".to_string()
}
fn default_text_shadow_opacity() -> f64 {
    0.5
}
fn default_background_color() -> String {
    "#000000".to_string()
}
fn default_background_opacity() -> f64 {
    0.05
}
fn default_notch_mode() -> bool {
    false
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            font_color: default_font_color(),
            font_family: default_font_family(),
            font_size: default_font_size(),
            opacity: default_opacity(),
            text_shadow: default_text_shadow(),
            text_shadow_color: default_text_shadow_color(),
            text_shadow_opacity: default_text_shadow_opacity(),
            background_color: default_background_color(),
            background_opacity: default_background_opacity(),
            notch_mode: default_notch_mode(),
            notch_monitor: None,
        }
    }
}

const SETTINGS_KEY: &str = "widget_settings";

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    let store = app.store("settings.bin").map_err(|e| e.to_string())?;
    match store.get(SETTINGS_KEY) {
        Some(val) => {
            serde_json::from_value(val).map_err(|e| format!("failed to parse settings: {e}"))
        }
        None => Ok(Settings::default()),
    }
}

#[tauri::command]
pub fn set_settings(
    app: AppHandle,
    settings: Settings,
    monitor_cache: State<'_, MonitorCache>,
) -> Result<(), String> {
    let store = app.store("settings.bin").map_err(|e| e.to_string())?;
    store.set(SETTINGS_KEY, serde_json::to_value(&settings).unwrap());
    let _ = store.save();

    // Emit to main window so it updates live
    let _ = app.emit("settings-update", &settings);

    // Apply or remove notch-mode window configuration
    if let Some(window) = app.get_webview_window("main") {
        if settings.notch_mode {
            let _ = crate::notch::enable_notch_mode(
                &window,
                settings.notch_monitor.as_deref(),
                &monitor_cache,
            );
        } else {
            let _ = crate::notch::disable_notch_mode(&window);
        }
    }

    Ok(())
}
