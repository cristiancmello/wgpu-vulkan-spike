use std::fs;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use wgpu_vulkan_spike::draw_command::DrawCommand;

#[test]
fn client_sends_draw_triangle_command_appears_in_buffer() {
  let socket_path = "/tmp/wgpu-draw.sock";

  if fs::metadata(socket_path).is_ok() {
      fs::remove_file(socket_path).expect("failed to remove old socket");
  }

  let buffer = Arc::new(Mutex::new(Vec::new()));
  let buffer_clone = Arc::clone(&buffer);

  let _server_thread = thread::spawn(move || {
      wgpu_vulkan_spike::socket_listener::spawn_socket_listener(socket_path, buffer_clone)
          .expect("failed to spawn socket listener");
  });

  thread::sleep(Duration::from_millis(100));

  let mut stream = UnixStream::connect(socket_path).expect("failed to connect to socket");
  stream.write_all(b"(draw-triangle 0.0 0.5 -0.5 -0.5 0.5 -0.5 1.0 0.0 0.0 1.0)\n").expect("failed to write");

  thread::sleep(Duration::from_millis(100));

  let locked_buffer = buffer.lock().unwrap();
  assert_eq!(locked_buffer.len(), 1);

  let expected = DrawCommand::DrawTriangle {
      x1: 0.0,
      y1: 0.5,
      x2: -0.5,
      y2: -0.5,
      x3: 0.5,
      y3: -0.5,
      r: 1.0,
      g: 0.0,
      b: 0.0,
      a: 1.0,
  };

  assert_eq!(locked_buffer[0], expected);
}
