struct TerrainUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    fog_color: vec4<f32>,
    fog_params: vec4<f32>,
    sun_direction: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: TerrainUniforms;

@group(1) @binding(0)
var atlas_texture: texture_2d<f32>;

@group(1) @binding(1)
var atlas_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) light: vec2<f32>,
    @location(3) ao: f32,
    @location(4) color: vec3<f32>,
    @location(5) normal_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) light: vec2<f32>,
    @location(2) ao: f32,
    @location(3) color: vec3<f32>,
    @location(4) normal_index: u32,
    @location(5) world_dist: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    out.light = in.light;
    out.ao = in.ao;
    out.color = in.color;
    out.normal_index = in.normal_index;
    out.world_dist = distance(in.position, uniforms.camera_pos.xyz);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_color: vec4<f32> = textureSample(atlas_texture, atlas_sampler, in.uv);

    // Alpha test for cutout transparent blocks
    if (tex_color.a < 0.1) {
        discard;
    }

    // Multiply texture by color tint (foliage / water)
    var base_color: vec3<f32> = tex_color.rgb * in.color;

    // Face directional shading
    var face_shade: f32 = 1.0;
    if (in.normal_index == 0u) {
        face_shade = 1.0;
    } else if (in.normal_index == 1u) {
        face_shade = 0.55;
    } else if (in.normal_index == 2u || in.normal_index == 3u) {
        face_shade = 0.8;
    } else {
        face_shade = 0.65;
    }

    // Smooth natural lighting calculation
    let sky_illumination = vec3<f32>(1.0, 1.0, 1.0) * in.light.y;
    let block_illumination = vec3<f32>(1.0, 0.82, 0.55) * in.light.x;
    let ambient_light = vec3<f32>(0.22, 0.22, 0.24);

    let light_sum = max(ambient_light, sky_illumination + block_illumination * 0.9);
    let ao_factor = mix(0.60, 1.0, in.ao);
    let total_light = clamp(light_sum, vec3<f32>(0.20), vec3<f32>(1.0)) * ao_factor * face_shade;
    var final_color = base_color * total_light;


    // Fog blending
    let fog_factor = clamp((in.world_dist - uniforms.fog_params.x) / max(uniforms.fog_params.y - uniforms.fog_params.x, 1.0), 0.0, 1.0);
    final_color = mix(final_color, uniforms.fog_color.rgb, fog_factor);

    return vec4<f32>(final_color, tex_color.a);
}
