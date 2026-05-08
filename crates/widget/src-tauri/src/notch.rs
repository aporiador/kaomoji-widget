use tauri::{LogicalPosition, LogicalSize, WebviewWindow};

use crate::monitor::MonitorCache;

/// Snap the window into the notch gutter with proper macOS window behaviour.
///
/// Window properties set via AppKit private API:
/// - `level` → NSStatusWindowLevel (25) so it sits above normal windows.
/// - `collectionBehavior` → canJoinAllSpaces | stationary | ignoresCycle | fullScreenNone.
/// - `movable` → false (locked in place).
/// - `hasShadow` → false (cleaner look in the menu-bar area).
pub fn enable_notch_mode(
    window: &WebviewWindow,
    monitor_name: Option<&str>,
    monitor_cache: &MonitorCache,
) -> Result<(), Box<dyn std::error::Error>> {
    let monitor = {
        let cache = monitor_cache.0.lock().unwrap();
        match monitor_name {
            Some(name) => cache.values().find(|n| n.name == name).cloned(),
            None => cache.values().next().cloned(),
        }
    }
    .ok_or("no monitor found")?;

    let has_notch = monitor.insets.top > 0.0;

    // Choose a width that spans the notch plus ~10 px padding on each side.
    // 14" MBP (≈1512 pt) → ~180 pt; 16" MBP (≈1728 pt) → ~220 pt.
    let window_width = if monitor.width > 1600.0 { 220.0 } else { 220.0 };

    // Height covers the entire notch gutter plus the original widget area.
    // If the monitor has no physical notch, keep the original compact height
    // so the widget does not look like an empty black bar.
    let window_height = if has_notch {
        monitor.insets.top + 50.0 + 2.0
    } else {
        50.0 + 2.0
    };

    let x = monitor.x + (monitor.width - window_width) / 2.0;
    let y = monitor.y;

    window.set_size(LogicalSize::new(window_width, window_height))?;
    window.set_skip_taskbar(true)?;

    #[cfg(target_os = "macos")]
    {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        let ns_window = window.ns_window()? as *mut AnyObject;
        unsafe {
            // NSStatusWindowLevel = 25 — must be raised *before* positioning
            // so macOS does not constrain the window to stay below the menu bar.
            let _: () = msg_send![ns_window, setLevel: 25_i32];

            // NSWindowCollectionBehavior:
            //   canJoinAllSpaces = 1 << 0
            //   stationary       = 1 << 4
            //   ignoresCycle     = 1 << 5
            //   fullScreenNone   = 1 << 12
            let behavior: u64 = (1 << 0) | (1 << 4) | (1 << 5) | (1 << 12);
            let _: () = msg_send![ns_window, setCollectionBehavior: behavior];

            let _: () = msg_send![ns_window, setMovable: false];
            let _: () = msg_send![ns_window, setHasShadow: false];
        }
    }

    // Position *after* the window level is raised so we can sit flush with
    // the top of the screen, covering the notch area.
    window.set_position(tauri::Position::Logical(LogicalPosition::new(x, y)))?;

    #[cfg(not(target_os = "macos"))]
    {
        window.set_always_on_top(true)?;
    }

    Ok(())
}

/// Restore the window to floating-widget mode.
pub fn disable_notch_mode(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    window.set_size(LogicalSize::new(200.0, 80.0))?;
    window.set_always_on_top(true)?;

    #[cfg(target_os = "macos")]
    {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        let ns_window = window.ns_window()? as *mut AnyObject;
        unsafe {
            // Reset to normal window level (NSNormalWindowLevel = 0)
            let _: () = msg_send![ns_window, setLevel: 0_i32];

            // Reset collection behavior to default (0)
            let _: () = msg_send![ns_window, setCollectionBehavior: 0_u64];

            let _: () = msg_send![ns_window, setMovable: true];
            let _: () = msg_send![ns_window, setHasShadow: true];
        }
    }

    Ok(())
}
