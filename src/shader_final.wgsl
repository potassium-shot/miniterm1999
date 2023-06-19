// // // VERTEX

struct VertexInput {
	@location(0) position: vec3<f32>,
}

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
  @location(0) position: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = vec4(in.position, 1.0);
  out.position = (in.position + 1.0) * 0.5;
  out.position.y = 1.0 - out.position.y;
	return out;
}

// // // FRAGMENT

@group(0) @binding(0)
var t_base: texture_2d<f32>;
@group(0) @binding(1)
var s_base: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return textureSample(t_base, s_base, in.position.xy);
}
