@group(0) @binding(0) var<uniform> mvp: MVP;
@group(0) @binding(1) var txSampler: sampler;
@group(0) @binding(2) var texture1: texture_2d<f32>;
@group(0) @binding(3) var texture2: texture_2d<f32>;

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
  let mvp_mat = mvp.proj * mvp.view * mvp.model;
  out.pos = mvp_mat * vec4f(input.pos, 1.0);
  out.uv = input.uv;
  out.normal = (mvp.model * vec4f(input.normal, 0.0)).xyz;
  return out;
}

@fragment
fn fragmentMain(input: VertOut) -> @location(0) vec4f {
  let n = (1.0 + input.normal) / 2.0;
  var tx1 = textureSample(texture1, txSampler, input.uv);
  var tx2 = textureSample(texture2, txSampler, input.uv);
  // draw normal instead of texture if alpha < 0.0001
  tx1 = mix(tx1, vec4f(n, 1.0), step(tx1.a, 0.0001));
  // mix tx1 and tx2, increasing tx2 influence based on alpha
  return mix(tx1 + tx2, tx2, tx2.a);
}