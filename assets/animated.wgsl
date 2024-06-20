const MAX_JOINTS = 5;

@group(0) @binding(0) var<uniform> mvp: MVP;
@group(0) @binding(1) var txSampler: sampler;
@group(0) @binding(2) var texture1: texture_2d<f32>;
@group(0) @binding(3) var texture2: texture_2d<f32>;
@group(0) @binding(4) var<uniform> joint_transforms: array<mat4x4<f32>, MAX_JOINTS>;

struct MVP {
  model: mat4x4<f32>,
  view: mat4x4<f32>,
  proj: mat4x4<f32>,
}

struct VertIn {
  @location(0) pos: vec3f,
  @location(1) uv: vec2f,
  @location(2) normal: vec3f,
  @location(3) joints: vec4u,
  @location(4) weights: vec4f,
}

struct VertOut {
  @builtin(position) pos: vec4f,
  @location(0) uv: vec2f,
  @location(1) normal: vec3f,
}

@vertex
fn vertexMain(input: VertIn) -> VertOut {
  var local_pos = vec4f(input.pos, 1.0);
  var local_norm = vec4f(input.normal, 0.0);
  // calculate bone transforms in local space
  for (var i = 0; i < 4; i++) {
    // position avg
    let pos_t = joint_transforms[input.joints[i]] * vec4f(input.pos, 1.0);
    local_pos = local_pos + input.weights[i] * pos_t;
    // normal avg
    let norm_t = joint_transforms[input.joints[i]] * vec4f(input.normal, 0.0);
    local_norm = local_norm + input.weights[i] * norm_t;
  }
  // mvp transform
  var out: VertOut;
  let mvp_mat = mvp.proj * mvp.view * mvp.model;
  out.pos = mvp_mat * local_pos;
  out.uv = input.uv;
  out.normal = (mvp.model * local_norm).xyz;
  return out;
}

@fragment
fn fragmentMain(input: VertOut) -> @location(0) vec4f {
  let n = (1 + input.normal) / 2;
  return vec4f(n, 1.0);
}