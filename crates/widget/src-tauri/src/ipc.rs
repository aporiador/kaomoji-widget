use crate::commands::dispatch;
use interprocess::local_socket::{prelude::*, GenericFilePath, ListenerOptions};
use tauri::AppHandle;

pub fn run_ipc_listener(app: AppHandle) {
    let socket_path = ipc_protocol::socket_path();

    #[cfg(unix)]
    if socket_path.exists() {
        if let Err(e) = std::fs::remove_file(&socket_path) {
            eprintln!("Warning: failed to remove stale socket {socket_path:?}: {e}");
        }
    }

    let name = match socket_path.clone().to_fs_name::<GenericFilePath>() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Invalid socket path {socket_path:?}: {e}");
            return;
        }
    };

    let listener = match ListenerOptions::new().name(name).create_sync() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind IPC socket at {socket_path:?}: {e}");
            return;
        }
    };

    for result in listener {
        match result {
            Ok(stream) => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn_blocking(move || {
                    handle_connection(stream, app_handle);
                });
            }
            Err(e) => {
                eprintln!("IPC accept error: {e}");
            }
        }
    }
}

fn handle_connection(mut stream: LocalSocketStream, app: AppHandle) {
    while let Ok(cmd) = ipc_protocol::read_message::<_, ipc_protocol::Command>(&mut stream) {
        let resp = dispatch(cmd, &app);
        if ipc_protocol::write_message(&mut stream, &resp).is_err() {
            break;
        }
    }
}
