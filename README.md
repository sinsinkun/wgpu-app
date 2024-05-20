# Wgpu App Base

Scaffolding for creating a general native app using a wgpu renderer.
Rendering logic and window handling are separated from app logic,
allowing for better compartmentalization.

Built-in support for common rendering utilities like MVP matrix, camera control,
MSAA filtering, z-buffer, texture render target, basic shape primitives, and more. 

Note: does not support compiling to wasm for browsers

<img src="assets/screenshot.png" width="700px" />

## Installation

Rust version: 1.76.0

`cargo build`/`cargo run`

### To-do:
- More primitives
- Custom uniforms for shaders
- Audio
- Model importing
