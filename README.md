# wgpu-vulkan-spike

A WebGPU/Vulkan rendering spike using Rust and wgpu.

## Prerequisites

- Rust 1.96 or later
- pkg-config

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

This will open a window with a green rendering surface powered by wgpu.

![window with a green rendering surface](docs/img/green-screen-rendering.png)

## Tests

Run all tests (unit + acceptance):

```bash
cargo test
```

Run only acceptance tests (E2E):

```bash
cargo test acceptance
```

Run tests with output:

```bash
cargo test -- --nocapture
```

Run a specific test:

```bash
cargo test client_sends_draw_triangle_command_appears_in_buffer
```