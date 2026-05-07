// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Set the activation policy before Tauri initialises NSApplication so the
    // dock icon never appears (not even briefly).  Policy 1 corresponds to
    // NSApplicationActivationPolicyAccessory.
    #[cfg(target_os = "macos")]
    unsafe {
        use objc2::runtime::AnyObject;
        use objc2::{class, msg_send};
        let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
        let _: bool = msg_send![ns_app, setActivationPolicy: 1_i64];
    }

    tauri_app_lib::run();
}
