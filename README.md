# Claude has a face on your desktop now

A little widget floats on your desktop. Claude picks the face based on what's happening in the conversation - happy, confused, smug, defeated - and updates it whenever it wants.

Idea comes from [here](https://eriskii.net/projects/claude-faces)

https://github.com/user-attachments/assets/7dc4fe3e-50a5-4796-ba21-b6a526dca7fd

## Features

- **Transparent window** - floats on your desktop above other windows
- **Draggable** - click and drag the kaomoji to move the widget
- **System tray** - show/hide, reset position, or quit from the tray icon
- **Image support** - display PNG, GIF, WebP, or JPG images
- **MCP bridge** - Claude Desktop / Claude Code or any other harness can control the widget via MCP tools

## Prerequisites

- [Rust](https://rustup.rs/)
- [Bun](https://bun.sh/)

**Platform-specific:**

- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Linux**: A compositor is required for transparency to work. On X11 without a compositor, the background will be black instead of transparent.
- **Windows**: Nothing extra needed.

Tested on macOS. Should work on Windows and Linux but I haven't tried - please file an issue if it breaks.

## Running

```bash
git clone <repo-url>
cd kaomoji-widget

bun install --cwd crates/widget
bun run build
```

macOS: `bun run build` will open an install window, after installing the widget is available in Applications.

```bash
./target/release/kaomoji-widget
```

## Claude Desktop Configuration

Add the MCP bridge to your Claude Desktop config file:

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "kaomoji": {
      "command": "/absolute/path/to/target/release/mcp-bridge"
    }
  }
}
```

To use your own image assets with the `set_asset` / `list_assets` tools, set `KAOMOJI_ASSETS_DIR` to any folder of images:

```json
{
  "mcpServers": {
    "kaomoji": {
      "command": "/absolute/path/to/target/release/mcp-bridge",
      "env": {
        "KAOMOJI_ASSETS_DIR": "/absolute/path/to/your/images"
      }
    }
  }
}
```

The `assets/kaomoji-pack/` folder in this repo contains a small example pack you can use or replace.

**Restart Claude Desktop** after adding the config. You should see the kaomoji tools available in Claude's tool list.

### Project instructions

To avoid telling Claude about the tool each conversation, you can use a project to set custom instructions.
Example:

> there's a kaomoji tool available - a kaomoji widget in the user's dock that's always visible - like a status light or a face. whatever you set stays set until you change it or clear it. you can update it whenever, including multiple times in one response. think of it as your continuous ambient presence, not a per-message reaction. if you don't update it, the last one you set is still showing.

## System tray

A tray icon appears in your system tray / menu bar:

- **Show** - bring the widget window to the front
- **Hide** - hide the widget window (the process keeps running)
- **Settings** - open settings menu
- **Reset Position** - move the widget back to its default position and forget the saved spot
- **Quit** - exit the widget entirely

If the widget ever seems to disappear, check the tray icon, it may just be hidden.

## Architecture

```
┌─────────────────┐   stdio    ┌──────────────┐  unix socket /  ┌────────────────┐
│ Claude Desktop  │ ─────────► │ mcp-bridge   │ ───named pipe──►│ kaomoji-widget │
│  (MCP client)   │            │  (bridge)    │                 │   (Tauri GUI)  │
└─────────────────┘            └──────────────┘                 └────────────────┘
```

There are two processes:

1. **Widget app** (`kaomoji-widget`) - Tauri 2 app. Long-running. Renders the borderless transparent window. Owns the display state. Listens on a local Unix domain socket (macOS/Linux) or named pipe (Windows) for update commands.
2. **MCP bridge** (`mcp-bridge`) - Tiny standalone Rust binary using `rmcp` with stdio transport. Claude Desktop spawns this on demand. Each tool call connects to the widget's local socket, sends a JSON command, and returns success/failure.

### Available MCP tools

- `set_kaomoji(text: string)` - change the displayed kaomoji text
- `set_image(path: string)` - display an image by absolute path (PNG, GIF, WebP, or JPG)
- `set_asset(name: string)` - display an image by file name from the configured assets directory
- `list_assets()` - list available image file names in the configured assets directory
- `clear()` - clear the display
- `is_running()` - check whether the widget is reachable

## Development

### Quick IPC test (no MCP needed)

```bash
# For development with hot reload:
bun run dev

# With the widget running, send a kaomoji directly:
cargo run --example send -p ipc-protocol -- "(▰˘◡˘▰)"

# Or use the convenience wrapper:
bun run send "(▰˘◡˘▰)"
```

## License

MIT
