//! IPC protocol for kaomoji-widget.
//!
//! Wire format: length-prefixed JSON.
//! Each message is a 4-byte big-endian u32 length, followed by that many bytes of UTF-8 JSON.
//! This crate provides shared types and framing helpers so both the widget and the MCP bridge
//! agree on the format without reimplementing it.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::path::PathBuf;

/// Commands sent from the MCP bridge → widget.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Command {
    SetKaomoji { text: String },
    SetImage { path: PathBuf },
    SetAnimation { frames: Vec<Frame>, fps: u32 },
    Clear,
    Ping,
}

/// A single frame in an animation sequence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Frame {
    pub path: PathBuf,
    pub duration_ms: u32,
}

/// Responses sent from the widget → MCP bridge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Response {
    Ok,
    Error { message: String },
    Pong,
}

/// Returns the platform-specific socket / named-pipe path used by both sides.
pub fn socket_path() -> PathBuf {
    #[cfg(unix)]
    {
        if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_dir).join("kaomoji-widget.sock")
        } else {
            let uid = nix::unistd::Uid::current();
            PathBuf::from(format!("/tmp/kaomoji-widget-{}.sock", uid))
        }
    }
    #[cfg(windows)]
    {
        PathBuf::from(r"\\.\pipe\kaomoji-widget")
    }
}

/// Serialize `msg` as JSON and write it to `w` with a 4-byte big-endian length prefix.
pub fn write_message<W: Write, T: Serialize>(w: &mut W, msg: &T) -> io::Result<()> {
    let json_bytes = serde_json::to_vec(msg).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, e)
    })?;
    let len = json_bytes.len() as u32;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(&json_bytes)?;
    w.flush()?;
    Ok(())
}

/// Read a length-prefixed JSON message from `r` and deserialize it into `T`.
pub fn read_message<R: Read, T: DeserializeOwned>(r: &mut R) -> io::Result<T> {
    let mut len_bytes = [0u8; 4];
    r.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    // Sanity cap to avoid unbounded allocation on corrupt streams.
    const MAX_MESSAGE_SIZE: usize = 64 * 1024 * 1024; // 64 MiB
    if len > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("message length {} exceeds maximum {}", len, MAX_MESSAGE_SIZE),
        ));
    }

    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    let msg = serde_json::from_slice(&buf).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, e)
    })?;
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn roundtrip_set_kaomoji() {
        let original = Command::SetKaomoji {
            text: "(▰˘◡˘▰)".into(),
        };
        let mut buf = Vec::new();
        write_message(&mut buf, &original).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded: Command = read_message(&mut cursor).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_set_image() {
        let original = Command::SetImage {
            path: PathBuf::from("/tmp/test.png"),
        };
        let mut buf = Vec::new();
        write_message(&mut buf, &original).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded: Command = read_message(&mut cursor).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_set_animation() {
        let original = Command::SetAnimation {
            frames: vec![
                Frame {
                    path: PathBuf::from("/tmp/frame1.png"),
                    duration_ms: 100,
                },
                Frame {
                    path: PathBuf::from("/tmp/frame2.png"),
                    duration_ms: 200,
                },
            ],
            fps: 10,
        };
        let mut buf = Vec::new();
        write_message(&mut buf, &original).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded: Command = read_message(&mut cursor).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_clear() {
        let original = Command::Clear;
        let mut buf = Vec::new();
        write_message(&mut buf, &original).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded: Command = read_message(&mut cursor).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_ping() {
        let original = Command::Ping;
        let mut buf = Vec::new();
        write_message(&mut buf, &original).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded: Command = read_message(&mut cursor).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_response_variants() {
        for original in [
            Response::Ok,
            Response::Error {
                message: "something went wrong".into(),
            },
            Response::Pong,
        ] {
            let mut buf = Vec::new();
            write_message(&mut buf, &original).unwrap();

            let mut cursor = Cursor::new(buf);
            let decoded: Response = read_message(&mut cursor).unwrap();
            assert_eq!(original, decoded);
        }
    }

    #[test]
    fn read_message_rejects_oversized_length() {
        let buf = vec![0xff; 4]; // length = u32::MAX
        let mut cursor = Cursor::new(buf);
        let result: Result<Command, _> = read_message(&mut cursor);
        assert!(result.is_err());
    }
}
