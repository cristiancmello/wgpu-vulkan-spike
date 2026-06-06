use std::sync::{Arc, Mutex};
use std::os::unix::net::UnixListener;
use std::fs;
use std::io::{BufRead, BufReader};
use crate::draw_command::{DrawCommand, parse_command};

pub fn spawn_socket_listener(
    socket_path: &str,
    buffer: Arc<Mutex<Vec<DrawCommand>>>,
) -> std::io::Result<()> {
    if fs::metadata(socket_path).is_ok() {
        fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;

    for stream in listener.incoming() {
        if let Ok(conn) = stream {
            let mut reader = BufReader::new(conn);
            let mut line = String::new();

            // Read line by line while connection is open
            while reader.read_line(&mut line).is_ok() && !line.is_empty() {
                if let Some(cmd) = parse_command(&line) {
                    buffer.lock().unwrap().push(cmd);
                }
                line.clear();
            }
        }
    }

    Ok(())
}
