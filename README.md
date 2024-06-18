# Wgpu App Base

Scaffolding for creating a general native app using a wgpu renderer.
Rendering logic and window handling are separated from app logic,
allowing for better compartmentalization.

Built-in support for common rendering utilities like MVP matrix, camera control,
MSAA filtering, z-buffer, texture render target, basic shape primitives, and more. 

Note 1: does not support compiling to wasm for browsers

Note 2: custom uniforms need to be converted to raw `&[u8]` byte data for consumption,
with a max size of 256 bytes (equivalent to 64 f32 values)

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
  - optional vertex indexing
  - WGSL instancing with `@builtin(instance_index) idx: u32`
  - resize responsive
  - supports transparency
  - supports rendering to texture
  - supports custom additional uniforms
  - MSAA enabled by default
  - depth buffer z-indexing enabled by default
  - MVP transforms pre-built
  - .obj model importing
- Text renderer built on top of custom renderer
- Input handler middleware interface
  - supports key binding

### Known Issues:

### To-do:
- .gltf file model importing
- Forward render lighting pass
- Shadow rendering
- Physics
- Audio
- Model importing
