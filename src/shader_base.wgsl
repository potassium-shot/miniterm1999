// // // VERTEX

struct VertexInput {
	@location(0) position: vec3<f32>,
}

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = vec4(in.position, 1.0);
	return out;
}

// // // FRAGMENT

const CHARACTER_BUFFER_WIDTH: u32 = 1024u;
const CHARACTER_WIDTH: f32 = 10.0;
const CHARACTER_HEIGHT: f32 = 10.0;
const FONT_TEXTURE_ROW: u32 = 16u;
const FONT_TEXTURE_WIDTH: f32 = 160.0;
const FONT_TEXTURE_HEIGHT: f32 = 160.0;

struct Character {
	@location(0) bg: vec3<f32>,
	@location(1) fg: vec3<f32>,
	@location(2) code: u32,
}

@group(0) @binding(0)
var t_font: texture_2d<f32>;
@group(0) @binding(1)
var s_font: sampler;

@group(1) @binding(0)
var<storage, read> characters: array<Character>;
@group(1) @binding(1)
var<uniform> scale_factor: f32;

fn get_character(x: u32, y: u32) -> Character {
	return characters[y * CHARACTER_BUFFER_WIDTH + x];
}

fn get_tex_coord(code: u32) -> vec2<f32> {
	return vec2(f32(code % FONT_TEXTURE_ROW), f32(code / FONT_TEXTURE_ROW)) * vec2(CHARACTER_WIDTH, CHARACTER_HEIGHT);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let char: Character = get_character(
		u32(floor(in.clip_position.x / CHARACTER_WIDTH * scale_factor)),
		u32(floor(in.clip_position.y / CHARACTER_HEIGHT * scale_factor))
	);

	if textureSample(t_font, s_font,
		(
			get_tex_coord(char.code) +
			vec2((in.clip_position.x * scale_factor) % CHARACTER_WIDTH, (in.clip_position.y * scale_factor) % CHARACTER_HEIGHT)
		) / vec2(FONT_TEXTURE_WIDTH, FONT_TEXTURE_HEIGHT)
	).r > 0.5
	{
		return vec4(char.fg, 1.0);
	}
	else
	{
		return vec4(char.bg, 1.0);
	}
}
