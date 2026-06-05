use std::sync::{Arc, Mutex};
use std::os::unix::net::UnixListener;
use std::fs;
use std::io::Read;
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
        if let Ok(mut conn) = stream {
            let mut buf = [0; 256];
            if let Ok(n) = conn.read(&mut buf) {
                let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                if let Some(cmd) = parse_command(&msg) {
                    buffer.lock().unwrap().push(cmd);
                }
            }
        }
        break;
    }
    
    Ok(())
}
