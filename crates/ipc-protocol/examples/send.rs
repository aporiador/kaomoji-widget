use interprocess::local_socket::{prelude::*, GenericFilePath, Stream};
use ipc_protocol::{Command, Response};
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    let cmd = if args.len() >= 3 && args[1] == "--image" {
        let path = PathBuf::from(&args[2]);
        Command::SetImage { path }
    } else {
        let text = args.get(1).cloned().unwrap_or_else(|| "(▰˘◡˘▰)".to_string());
        Command::SetKaomoji { text }
    };

    let socket_path = ipc_protocol::socket_path();

    let name = socket_path
        .to_fs_name::<GenericFilePath>()
        .expect("invalid socket path");

    let mut stream = Stream::connect(name)
        .expect("failed to connect to widget socket; is the widget running?");

    ipc_protocol::write_message(&mut stream, &cmd).expect("failed to write command");

    let resp: Response = ipc_protocol::read_message(&mut stream).expect("failed to read response");
    match resp {
        Response::Ok => println!("Widget updated successfully."),
        Response::Error { message } => eprintln!("Widget returned error: {}", message),
        Response::Pong => println!("Pong (unexpected)."),
    }
}
