@group(0) @binding(0) var<uniform> mvp: MVP;
@group(0) @binding(1) var txSampler: sampler;
@group(0) @binding(2) var texture1: texture_2d<f32>;
@group(0) @binding(3) var texture2: texture_2d<f32>;

@group(1) @binding(0) var<uniform> win_size: vec2f;

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
}

@vertex
fn vertexMain(input: VertIn) -> VertOut {
  var out: VertOut;
  let mvpMat = mvp.proj * mvp.view * mvp.model;
  out.pos = mvpMat * vec4f(input.pos, 1.0);
  out.uv = input.uv;
  out.normal = (mvp.model * vec4f(input.normal, 0.0)).xyz;
  return out;
}

@fragment
fn fragmentMain(input: VertOut) -> @location(0) vec4f {
  var out = textureSample(texture1, txSampler, input.uv);
  let y_border = 0.015;
  let x_border = y_border * win_size.y / win_size.x;
  if (input.uv.x < x_border || input.uv.x > 1.0 - x_border || input.uv.y < y_border || input.uv.y > 1.0 - y_border) {
    out = vec4f(0.8, 0.0, 0.0, 1.0);
  }
  return out;
}