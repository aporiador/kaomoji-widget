use std::path::PathBuf;
use std::time::Duration;

use rmcp::{
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
    ServerHandler, ServiceExt,
};
use tokio::time::timeout;

use ipc_protocol::{Command, Response};

#[derive(Debug, Clone)]
struct BridgeServer {
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SetKaomojiRequest {
    #[schemars(description = "The kaomoji text to display in the widget")]
    text: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SetImageRequest {
    #[schemars(description = "Absolute path to the image file (PNG, GIF, WebP, or JPG)")]
    path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SetAssetRequest {
    #[schemars(
        description = "File name of an asset in the configured assets directory (use list_assets to see available names)"
    )]
    name: String,
}

#[tool_router]
impl BridgeServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Set the kaomoji displayed in the widget")]
    async fn set_kaomoji(
        &self,
        request: Parameters<SetKaomojiRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let Parameters(SetKaomojiRequest { text }) = request;
        match send_command(Command::SetKaomoji { text }).await {
            Ok(Response::Ok) => Ok(CallToolResult::success(vec![Content::text(
                "Kaomoji updated successfully.",
            )])),
            Ok(Response::Error { message }) => Err(ErrorData::internal_error(message, None)),
            Ok(Response::Pong) => Ok(CallToolResult::success(vec![Content::text(
                "Unexpected pong response.",
            )])),
            Err(e) => Err(ErrorData::internal_error(e, None)),
        }
    }

    #[tool(description = "Set an image in the widget by absolute path (PNG, GIF, WebP, or JPG).")]
    async fn set_image(
        &self,
        request: Parameters<SetImageRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let Parameters(SetImageRequest { path }) = request;
        let path = PathBuf::from(path);
        match send_command(Command::SetImage { path }).await {
            Ok(Response::Ok) => Ok(CallToolResult::success(vec![Content::text(
                "Image updated successfully.",
            )])),
            Ok(Response::Error { message }) => Err(ErrorData::internal_error(message, None)),
            Ok(Response::Pong) => Ok(CallToolResult::success(vec![Content::text(
                "Unexpected pong response.",
            )])),
            Err(e) => Err(ErrorData::internal_error(e, None)),
        }
    }

    #[tool(
        description = "List image assets available in the configured assets directory (set via KAOMOJI_ASSETS_DIR env var)"
    )]
    async fn list_assets(&self) -> Result<CallToolResult, ErrorData> {
        match assets_dir() {
            Some(dir) => {
                let mut names = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                names.push(name.to_string());
                            }
                        }
                    }
                }
                names.sort();
                let text = if names.is_empty() {
                    "No assets found in the configured assets directory.".into()
                } else {
                    format!("Available assets:\n{}", names.join("\n"))
                };
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            None => Ok(CallToolResult::success(vec![Content::text(
                "No assets directory configured. Set KAOMOJI_ASSETS_DIR to point to your assets folder.",
            )])),
        }
    }

    #[tool(
        description = "Display an image asset by file name from the configured assets directory (set via KAOMOJI_ASSETS_DIR env var). Use list_assets to see available names."
    )]
    async fn set_asset(
        &self,
        request: Parameters<SetAssetRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let Parameters(SetAssetRequest { name }) = request;
        match resolve_asset_path(&name) {
            Some(path) => match send_command(Command::SetImage { path }).await {
                Ok(Response::Ok) => Ok(CallToolResult::success(vec![Content::text(
                    "Asset displayed successfully.",
                )])),
                Ok(Response::Error { message }) => Err(ErrorData::internal_error(message, None)),
                Ok(Response::Pong) => Ok(CallToolResult::success(vec![Content::text(
                    "Unexpected pong response.",
                )])),
                Err(e) => Err(ErrorData::internal_error(e, None)),
            },
            None => Err(ErrorData::internal_error(
                format!(
                    "Asset '{}' not found. Use list_assets to see available names.",
                    name
                ),
                None,
            )),
        }
    }

    #[tool(description = "Clear the widget display")]
    async fn clear(&self) -> Result<CallToolResult, ErrorData> {
        match send_command(Command::Clear).await {
            Ok(Response::Ok) => Ok(CallToolResult::success(vec![Content::text(
                "Display cleared.",
            )])),
            Ok(Response::Error { message }) => Err(ErrorData::internal_error(message, None)),
            Ok(Response::Pong) => Ok(CallToolResult::success(vec![Content::text(
                "Unexpected pong response.",
            )])),
            Err(e) => Err(ErrorData::internal_error(e, None)),
        }
    }

    #[tool(description = "Check whether the kaomoji widget is running")]
    async fn is_running(&self) -> Result<CallToolResult, ErrorData> {
        match send_command(Command::Ping).await {
            Ok(Response::Pong) => Ok(CallToolResult::success(vec![Content::text(
                "The kaomoji widget is running.",
            )])),
            Ok(Response::Ok) => Ok(CallToolResult::success(vec![Content::text(
                "The kaomoji widget is running (responded unexpectedly).",
            )])),
            Ok(Response::Error { message }) => {
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "The kaomoji widget is running but returned an error: {}",
                    message
                ))]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
        }
    }
}

#[tool_handler(
    instructions = "MCP bridge for the kaomoji widget. Use set_kaomoji to change the text, set_image to show an image by absolute path, set_asset to pick from your configured assets directory, list_assets to see what's available, clear to hide the widget, and is_running to check if the widget process is alive."
)]
impl ServerHandler for BridgeServer {}

/// Returns the assets directory to use for `list_assets` and `set_asset`.
///
/// Priority:
/// 1. `KAOMOJI_ASSETS_DIR` environment variable (explicit user config)
/// 2. `assets/kaomoji-pack` relative to the current working directory (dev convenience)
fn assets_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("KAOMOJI_ASSETS_DIR") {
        let path = PathBuf::from(dir);
        if path.exists() {
            return Some(path);
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        let path = cwd.join("assets").join("kaomoji-pack");
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn resolve_asset_path(name: &str) -> Option<PathBuf> {
    let dir = assets_dir()?;
    let path = dir.join(name);
    if path.exists() && path.is_file() {
        Some(path)
    } else {
        None
    }
}

async fn send_command(cmd: Command) -> Result<Response, String> {
    let path = ipc_protocol::socket_path();
    let result = timeout(
        Duration::from_secs(1),
        tokio::task::spawn_blocking(move || {
            use interprocess::local_socket::{prelude::LocalSocketStream, GenericFilePath, ToFsName};
            use interprocess::local_socket::traits::Stream;
            let name = path.to_fs_name::<GenericFilePath>()
                .map_err(|e| format!("Invalid socket path: {e}"))?;
            let mut conn = LocalSocketStream::connect(name)
                .map_err(|e| format!("kaomoji widget doesn't appear to be running. Start it with `kaomoji-widget`. ({e})",))?;
            ipc_protocol::write_message(&mut conn, &cmd)
                .map_err(|e| format!("Failed to send command: {e}"))?;
            let response: Response = ipc_protocol::read_message(&mut conn)
                .map_err(|e| format!("Failed to read response: {e}"))?;
            Ok(response)
        }),
    )
    .await;

    match result {
        Ok(Ok(Ok(resp))) => Ok(resp),
        Ok(Ok(Err(e))) => Err(e),
        Ok(Err(_)) => Err("spawn_blocking task panicked".into()),
        Err(_) => {
            Err("Connection to kaomoji widget timed out. Start it with `kaomoji-widget`.".into())
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // All logging MUST go to stderr; stdout is reserved for the JSON-RPC MCP protocol.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let server = BridgeServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
