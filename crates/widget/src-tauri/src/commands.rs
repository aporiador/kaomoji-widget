use serde_json::json;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

pub fn dispatch(cmd: ipc_protocol::Command, app: &AppHandle) -> ipc_protocol::Response {
    use ipc_protocol::{Command, Response};
    match cmd {
        Command::SetKaomoji { text, .. } => {
            let payload = json!({"kind": "kaomoji", "text": text});
            if let Err(e) = app.emit("display-update", payload) {
                Response::Error {
                    message: format!("failed to emit event: {e}"),
                }
            } else {
                Response::Ok
            }
        }
        Command::Clear => {
            let payload = json!({"kind": "empty"});
            if let Err(e) = app.emit("display-update", payload) {
                Response::Error {
                    message: format!("failed to emit event: {e}"),
                }
            } else {
                Response::Ok
            }
        }
        Command::Ping => Response::Pong,
        Command::SetImage { path } => handle_set_image(path, app),
        Command::SetAnimation { .. } => Response::Error {
            message: "not implemented".to_string(),
        },
    }
}

fn handle_set_image(path: PathBuf, app: &AppHandle) -> ipc_protocol::Response {
    use ipc_protocol::Response;

    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            return Response::Error {
                message: format!("failed to read image: {e}"),
            };
        }
    };

    let mime = match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => match ext.to_lowercase().as_str() {
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "jpg" | "jpeg" => "image/jpeg",
            _ => {
                return Response::Error {
                    message: format!("unsupported image format: {ext}"),
                };
            }
        },
        None => {
            return Response::Error {
                message: "image path has no extension".into(),
            };
        }
    };

    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    let data_url = format!("data:{};base64,{}", mime, b64);
    let payload = json!({"kind": "image", "data": data_url, "mime": mime});

    if let Err(e) = app.emit("display-update", payload) {
        Response::Error {
            message: format!("failed to emit event: {e}"),
        }
    } else {
        Response::Ok
    }
}
