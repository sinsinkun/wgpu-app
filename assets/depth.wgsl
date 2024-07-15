@group(0) @binding(0) var<uniform> mvp: MVP;

struct MVP {
  model: mat4x4<f32>,
  view: mat4x4<f32>,
  proj: mat4x4<f32>,
}

struct VertIn {
  @location(0) pos: vec3f,
  @location(1) uv: vec2f,
  @location(2) normal: vec3f,
}

struct VertOut {
  @builtin(position) pos: vec4f,
  @location(0) uv: vec2f,
  @location(1) normal: vec3f,
  @location(2) z: f32,
}

@vertex
fn vertexMain(input: VertIn) -> VertOut {
  var out: VertOut;
  let mvp_mat = mvp.proj * mvp.view * mvp.model;
  out.pos = mvp_mat * vec4f(input.pos, 1.0);
  out.uv = input.uv;
  out.normal = (mvp.model * vec4f(input.normal, 0.0)).xyz;
  out.z = out.pos.z / 1000.0;
  return out;
}

@fragment
fn fragmentMain(input: VertOut) -> @location(0) vec4f {
  return vec4f(vec3f(input.z), 1.0);
}