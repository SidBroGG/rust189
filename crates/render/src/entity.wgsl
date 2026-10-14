struct EntityUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    fog_color: vec4<f32>,
    fog_params: vec4<f32>,
    sun_direction: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: EntityUniforms;

@group(1) @binding(0)
var skin_texture: texture_2d<f32>;

@group(1) @binding(1)
var skin_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) world_dist: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    out.normal = in.normal;
    out.color = in.color;
    out.world_dist = distance(in.position, uniforms.camera_pos.xyz);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_color: vec4<f32> = textureSample(skin_texture, skin_sampler, in.uv);
    if (tex_color.a < 0.1) {
        discard;
    }

    var sun_dir = normalize(uniforms.sun_direction.xyz);
    var norm = normalize(in.normal);
    var diffuse: f32 = max(dot(norm, sun_dir), 0.0);
    var light: f32 = 0.55 + 0.45 * diffuse;

    var rgb = tex_color.rgb * in.color.rgb * light;

    var fog_start = uniforms.fog_params.x;
    var fog_end = uniforms.fog_params.y;
    var fog_factor: f32 = clamp((in.world_dist - fog_start) / max(fog_end - fog_start, 1.0), 0.0, 1.0);

    rgb = mix(rgb, uniforms.fog_color.rgb, fog_factor);
    return vec4<f32>(rgb, tex_color.a * in.color.a);
}
