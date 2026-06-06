# wgpu-vulkan-spike

A rendering server that accepts draw commands via Unix socket from external processes. Built with wgpu, winit, and Rust following Freeman/GOOS TDD methodology.

## Prerequisites

- Rust 1.96 or later
- pkg-config
- socat (for manual testing)

## Build

```bash
cargo build
```

For an optimized release build:

```bash
cargo build --release
```

## Run

```bash
cargo run
```

Or run the release build:

```bash
cargo run --release
```

A window will open showing the rendered output.

## Protocol

Commands are S-expressions sent line-by-line to `/tmp/wgpu-draw.sock`.

### Available Commands

**Clear background:**
```bash
printf '(clear r g b a)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
```

**Draw triangle:**
```bash
printf '(draw-triangle id x1 y1 x2 y2 x3 y3 r g b a)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
```

**Draw rectangle:**
```bash
printf '(draw-rect id x y w h r g b a)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
```

**Transform (translate + scale):**
```bash
printf '(set-transform id tx ty sx sy angle)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
```

**Reset buffer:**
```bash
printf '(reset)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
```

## Manual Tests

### Test 1: Single Triangle
```bash
# Terminal 1
cargo run

# Terminal 2
printf '(clear 0.0 0.0 0.0 1.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
printf '(draw-triangle 1 0.0 0.5 -0.5 -0.5 0.5 -0.5 1.0 0.0 0.0 1.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
```

**Expected**: Red triangle appears in center.

### Test 2: Triangle Translation
```bash
printf '(clear 0.0 0.0 0.0 1.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
printf '(draw-triangle 1 0.0 0.5 -0.5 -0.5 0.5 -0.5 1.0 0.0 0.0 1.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
printf '(set-transform 1 0.3 0.0 1.0 1.0 0.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
```

**Expected**: Red triangle moves 0.3 units right.

### Test 3: Triangle Scale
```bash
printf '(clear 0.0 0.0 0.0 1.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
printf '(draw-triangle 1 0.0 0.5 -0.5 -0.5 0.5 -0.5 1.0 0.0 0.0 1.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
printf '(set-transform 1 0.0 0.0 0.5 0.5 0.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
```

**Expected**: Red triangle becomes half size.

### Test 4: Multiple Primitives with Independent Transforms
```bash
printf '(clear 0.0 0.0 0.0 1.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock

# Draw 3 triangles
printf '(draw-triangle 1 0.0 0.5 -0.5 -0.5 0.5 -0.5 1.0 0.0 0.0 1.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
printf '(draw-triangle 2 -0.7 0.2 -1.0 -0.3 -0.4 -0.3 0.0 1.0 0.0 1.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
printf '(draw-triangle 3 0.7 0.2 0.4 -0.3 1.0 -0.3 0.0 0.0 1.0 1.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock

# Transform each independently
printf '(set-transform 1 0.3 0.0 1.0 1.0 0.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
printf '(set-transform 2 0.0 0.3 1.5 1.5 0.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
printf '(set-transform 3 0.0 -0.2 0.7 0.7 0.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
```

**Expected**: 
- Red triangle (ID=1) moves right
- Green triangle (ID=2) moves up and scales 1.5x
- Blue triangle (ID=3) moves down and scales 0.7x
- **Each transforms independently**

### Test 5: Persistent Buffer
```bash
printf '(clear 0.0 0.0 0.0 1.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
printf '(draw-triangle 1 0.0 0.5 -0.5 -0.5 0.5 -0.5 1.0 0.0 0.0 1.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
printf '(set-transform 1 0.2 0.0 1.0 1.0 0.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock

# Draw second triangle — first one remains visible
printf '(draw-triangle 2 -0.5 -0.5 -1.0 -0.3 -0.3 -0.3 0.0 1.0 0.0 1.0)\n' | socat - UNIX-CONNECT:/tmp/wgpu-draw.sock
```

**Expected**: Both triangles visible. Red triangle retains its transform.

## Automated Tests

Run all tests:

```bash
cargo test
```

Run only transform tests:

```bash
cargo test --test acceptance_transform
```

Run tests with output:

```bash
cargo test -- --nocapture
```

Run a specific test:

```bash
cargo test parse_set_transform_command
```

## Coordinates

- **NDC (Normalized Device Coordinates)**: X and Y range from -1.0 to 1.0
- **Y axis**: Grows upward (0.0 is center, 1.0 is top, -1.0 is bottom)
- **X axis**: Grows rightward (0.0 is center, 1.0 is right, -1.0 is left)
- **Colors**: RGBA, each component in [0.0, 1.0]

## Architecture

- `src/main.rs` - winit event loop
- `src/state.rs` - Rendering state management
- `src/renderer.rs` - Vertex struct and pipeline
- `src/draw_command.rs` - Command parser
- `src/socket_listener.rs` - Unix socket thread
- `src/transform.rs` - Transform matrices
- `src/primitive.rs` - Primitive buffer
- `src/shader.wgsl` - WGSL shaders