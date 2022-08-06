// Vertex shader

struct Camera {
  view_pos: vec4<f32>,
  view_proj: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
  @location(0) position: vec3<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
  model: VertexInput,
) -> VertexOutput {
  var out: VertexOutput;
  out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
  out.tex_coords = model.position.xz / 4.0;
  return out;
}

// Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
  let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);

  let result = object_color.xyz;

  return vec4<f32>(result, 1.0);
}

