use std::fs;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use wgpu_vulkan_spike::draw_command::DrawCommand;

#[test]
fn renderer_receives_and_processes_draw_triangle() {
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

#[test]
fn renderer_receives_clear_command() {
    let socket_path = "/tmp/wgpu-draw-clear.sock";

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
    stream.write_all(b"(clear 1.0 0.0 0.0 1.0)\n").expect("failed to write");

    thread::sleep(Duration::from_millis(100));

    let locked_buffer = buffer.lock().unwrap();
    assert_eq!(locked_buffer.len(), 1);

    let expected = DrawCommand::Clear {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    assert_eq!(locked_buffer[0], expected);
}

#[test]
fn renderer_receives_present_command() {
    let socket_path = "/tmp/wgpu-draw-present.sock";

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
    stream.write_all(b"(present)\n").expect("failed to write");

    thread::sleep(Duration::from_millis(100));

    let locked_buffer = buffer.lock().unwrap();
    assert_eq!(locked_buffer.len(), 1);
    assert_eq!(locked_buffer[0], DrawCommand::Present);
}

#[test]
fn renderer_receives_draw_rect_command() {
    let socket_path = "/tmp/wgpu-draw-rect.sock";

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
    stream.write_all(b"(draw-rect -0.5 0.5 1.0 1.0 0.0 1.0 0.0 1.0)\n").expect("failed to write");

    thread::sleep(Duration::from_millis(100));

    let locked_buffer = buffer.lock().unwrap();
    assert_eq!(locked_buffer.len(), 1);

    let expected = DrawCommand::DrawRect {
        x: -0.5,
        y: 0.5,
        w: 1.0,
        h: 1.0,
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    assert_eq!(locked_buffer[0], expected);
}

#[test]
fn renderer_receives_multiple_commands_same_connection() {
    let socket_path = "/tmp/wgpu-draw-multi.sock";

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
    // Send 3 commands in one connection
    stream.write_all(b"(clear 0.0 0.0 0.0 1.0)\n(draw-triangle 0.0 0.5 -0.5 -0.5 0.5 -0.5 1.0 0.0 0.0 1.0)\n(draw-rect -0.5 0.5 1.0 1.0 0.0 1.0 0.0 1.0)\n").expect("failed to write");

    thread::sleep(Duration::from_millis(100));

    let locked_buffer = buffer.lock().unwrap();
    assert_eq!(locked_buffer.len(), 3);

    // Verify Clear
    assert_eq!(locked_buffer[0], DrawCommand::Clear {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });

    // Verify DrawTriangle
    assert_eq!(locked_buffer[1], DrawCommand::DrawTriangle {
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
    });

    // Verify DrawRect
    assert_eq!(locked_buffer[2], DrawCommand::DrawRect {
        x: -0.5,
        y: 0.5,
        w: 1.0,
        h: 1.0,
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    });
}
