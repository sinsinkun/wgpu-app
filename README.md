# Wgpu App Base

Scaffolding for creating a general native app using a wgpu renderer.
Rendering logic and window handling are separated from app logic,
allowing for better compartmentalization.

Due to the nature of rust's restrictive type system, it is difficult to implement 
a custom solution for creating updateable uniform buffers in bind groups with dynamic 
allowances for the buffer data type (i.e. f32 array, f32 value, u32 value, custom struct). 

Built-in support for common rendering utilities like MVP matrix, camera control,
MSAA filtering, z-buffer, texture render target, basic shape primitives, and more. 

Note: does not support compiling to wasm for browsers

<img src="assets/screenshot.png" width="500px" />

Basic object rendering + MSAA + z-buffer sorting with text capabilities

<img src="assets/screenshot2.png" width="500px" />

Text blending on existing textures

## Installation

Rust version: 1.76.0

`cargo build`/`cargo run`

## Feature Set
- Winit setup independent from app structure
- Wgpu renderer setup independent from app structure
  - simplified pipeline setup
  - resize responsive
  - supports transparency
  - MSAA enabled by default
  - depth buffer z-indexing enabled by default
  - MVP transforms pre-built
  - supports rendering to texture
- Text renderer built on top of custom renderer
- Input handler middleware interface
  - supports key binding

### Known Issues:

### To-do:
- Forward render lighting pass
- Shadow rendering
- More primitives
- Physics
- Audio
- Model importing
