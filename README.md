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