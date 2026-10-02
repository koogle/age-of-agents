// Ground: generated painted textures per biome, blended across cell borders
// with a ragged edge, cel lit, under the shared fog of war.
@group(1) @binding(0) var ground_layers: texture_2d_array<f32>;
@group(1) @binding(1) var ground_index: texture_2d<f32>;
@group(1) @binding(2) var repeat_sampler: sampler;

const REPEAT: f32 = 2.5;
const BEACH: i32 = 8;

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) shade: f32,
};

@vertex
fn vs(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) shade: f32) -> VOut {
    var out: VOut;
    out.world = position;
    out.normal = normal;
    out.shade = shade;
    out.clip = g.view_proj * vec4<f32>(bend(position), 1.0);
    return out;
}

fn layer_at(cell: vec2<f32>) -> i32 {
    let c = clamp(cell, vec2<f32>(0.0), g.map_size - 1.0);
    return i32(round(textureLoad(ground_index, vec2<i32>(c), 0).r * 255.0));
}

// rgb = painted colour (linear), a = 1 where the biome is known.
fn paint_sample(xz: vec2<f32>, layer: i32) -> vec4<f32> {
    let known = select(1.0, 0.0, layer > 9);
    let l = min(layer, 9);
    let uv = xz / REPEAT;
    let a = textureSample(ground_layers, repeat_sampler, uv, l).rgb;
    let b = textureSample(ground_layers, repeat_sampler, mat2x2<f32>(0.8, -0.6, 0.6, 0.8) * uv * 0.43 + 0.37, l).rgb;
    let paint = mix(a, b, 0.3 * fbm(xz * 0.21));
    // Lift the brushwork that mipmapping flattens at gameplay distance.
    let mean = textureSampleLevel(ground_layers, repeat_sampler, uv, l, 9.0).rgb;
    return vec4<f32>(max(mean + (paint - mean) * 1.8, vec3<f32>(0.0)) * known, known);
}

fn ground_paint(xz: vec2<f32>) -> vec4<f32> {
    // Blend between simulation cells, which are finer than world units.
    let cell = xz / g.map_size * vec2<f32>(textureDimensions(ground_index));
    let gp = cell - 0.5;
    let base = floor(gp);
    var f = fract(gp) + (vec2<f32>(fbm(cell * 1.7), fbm(cell * 1.7 + 11.3)) - 0.5) * 0.7;
    f = smoothstep(vec2<f32>(0.3), vec2<f32>(0.7), clamp(f, vec2<f32>(0.0), vec2<f32>(1.0)));
    let c00 = paint_sample(xz, layer_at(base));
    let c10 = paint_sample(xz, layer_at(base + vec2<f32>(1.0, 0.0)));
    let c01 = paint_sample(xz, layer_at(base + vec2<f32>(0.0, 1.0)));
    let c11 = paint_sample(xz, layer_at(base + vec2<f32>(1.0, 1.0)));
    return mix(mix(c00, c10, f.x), mix(c01, c11, f.x), f.y);
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let xz = in.world.xz;
    let outside = max(max(-xz, xz - g.map_size), vec2<f32>(0.0));
    let wild = smoothstep(0.0, 1.6, length(outside));
    let uv = clamp(xz / g.map_size, vec2<f32>(0.0), vec2<f32>(1.0));
    let biome = srgb_to_linear(textureSampleLevel(cells, linear_clamp, uv, 0.0).rgb);
    var land = mix(biome, vec3<f32>(0.38, 0.55, 0.16), wild) * in.shade;
    let patch_tone = fbm(xz * 0.32);
    let painted = ground_paint(xz) * (1.0 - wild);
    land = land * (1.0 - painted.a) + painted.rgb * in.shade * mix(0.94, 1.06, patch_tone);
    let h = in.world.y;
    let beach = paint_sample(xz, BEACH);
    let color0 = mix(beach.rgb, land, smoothstep(-0.2, -0.08, h));
    var color = mix(color0 * vec3<f32>(0.35, 0.8, 0.85), color0, smoothstep(-0.57, -0.32, h));
    let foam = smoothstep(0.03, 0.0, abs(h + 0.32 - 0.012 * sin(g.time * 1.4 + xz.x * 2.0 + xz.y)));
    color = mix(color, vec3<f32>(1.0), foam * 0.85);
    color *= cel_light(normalize(in.normal));
    color = world_light(color, xz);
    return vec4<f32>(distance_fog(color, in.world), 1.0);
}
