struct TerrainUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    fog_color: vec4<f32>,
    fog_params: vec4<f32>,
    sun_direction: vec4<f32>,
    atlas_tile_size: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: TerrainUniforms;

@group(1) @binding(0)
var atlas_texture: texture_2d<f32>;

@group(1) @binding(1)
var atlas_sampler: sampler;

struct VertexInput {
    @location(0) packed: vec4<u32>,
    @location(1) chunk_offset: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) light: vec2<f32>,
    @location(2) ao: f32,
    @location(3) color: vec3<f32>,
    @location(4) normal_index: u32,
    @location(5) world_dist: f32,
    @location(6) local_uv: vec2<f32>,
    @location(7) is_greedy: u32,
    @location(8) is_cutout: u32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let d0 = in.packed.x;
    let d1 = in.packed.y;
    let d2 = in.packed.z;
    let d3 = in.packed.w;

    // Unpack local position: 10 bits each, 32 units per block (0..512 = 0.0..16.0)
    let px = f32(d0 & 1023u) / 32.0;
    let py = f32((d0 >> 10u) & 1023u) / 32.0;
    let pz = f32((d0 >> 20u) & 1023u) / 32.0;
    let ao_val = (d0 >> 30u) & 3u;
    let normal_idx = (d3 >> 17u) & 7u;

    let local_pos = vec3<f32>(px, py, pz);
    let world_pos = in.chunk_offset + local_pos;

    out.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 1.0);

    // Unpack atlas UV coordinates: 16 bits each (normalized 0..65535 -> 0.0..1.0)
    out.uv = vec2<f32>(
        f32(d1 & 65535u) / 65535.0,
        f32((d1 >> 16u) & 65535u) / 65535.0
    );

    // Unpack color tint RGB (8 bits each) and light (4 bits each)
    let cr = f32(d2 & 255u) / 255.0;
    let cg = f32((d2 >> 8u) & 255u) / 255.0;
    let cb = f32((d2 >> 16u) & 255u) / 255.0;
    out.color = vec3<f32>(cr, cg, cb);

    let bl = f32((d2 >> 24u) & 15u) / 15.0;
    let sl = f32((d2 >> 28u) & 15u) / 15.0;
    out.light = vec2<f32>(bl, sl);

    // Convert AO level (0..3) to factor: 0 -> 0.45, 1 -> 0.65, 2 -> 0.82, 3 -> 1.0
    if (ao_val == 0u) {
        out.ao = 0.45;
    } else if (ao_val == 1u) {
        out.ao = 0.65;
    } else if (ao_val == 2u) {
        out.ao = 0.82;
    } else {
        out.ao = 1.0;
    }

    out.normal_index = normal_idx;
    out.world_dist = distance(world_pos, uniforms.camera_pos.xyz);

    // Unpack greedy local uv & flag & cutout flag
    out.local_uv = vec2<f32>(
        f32(d3 & 255u),
        f32((d3 >> 8u) & 255u)
    );
    out.is_greedy = (d3 >> 16u) & 1u;
    out.is_cutout = (d3 >> 20u) & 1u;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_color: vec4<f32>;

    if (in.is_greedy != 0u) {
        let tile_uv = clamp(fract(in.local_uv), vec2<f32>(0.0), vec2<f32>(0.9999));
        let sample_uv = in.uv + tile_uv * uniforms.atlas_tile_size.xy;
        let ddx = dpdx(in.local_uv) * uniforms.atlas_tile_size.xy;
        let ddy = dpdy(in.local_uv) * uniforms.atlas_tile_size.xy;
        tex_color = textureSampleGrad(atlas_texture, atlas_sampler, sample_uv, ddx, ddy);
    } else {
        tex_color = textureSample(atlas_texture, atlas_sampler, in.uv);
    }

    // Alpha test only for cutout transparent blocks (leaves, tall grass, saplings, glass, etc.)
    // Solid opaque blocks never discard, eliminating pixel-sized transparent seams at block borders.
    if (in.is_cutout != 0u && tex_color.a < 0.1) {
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
