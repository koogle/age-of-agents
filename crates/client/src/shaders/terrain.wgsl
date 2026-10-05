// Ground: generated painted textures per biome, blended across cell borders
// with a ragged edge, cel lit, under the shared fog of war.
@group(1) @binding(0) var ground_layers: texture_2d_array<f32>;
@group(1) @binding(1) var ground_index: texture_2d<f32>;
@group(1) @binding(2) var repeat_sampler: sampler;

const REPEAT: f32 = 2.5;
const BEACH: i32 = 8;
// Ground-index values past the texture layers: rivers (painted with the
// shallows) and bare peaks (painted with the highland, then turned to rock).
const RIVER: i32 = 10;
const MOUNTAIN: i32 = 11;

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
    // Clamp in cells (the index texture), not world units: cells are half a unit.
    let c = clamp(cell, vec2<f32>(0.0), vec2<f32>(textureDimensions(ground_index)) - 1.0);
    return i32(round(textureLoad(ground_index, vec2<i32>(c), 0).r * 255.0));
}

// rgb = painted colour (linear), a = 1 where the biome is known.
fn paint_sample(xz: vec2<f32>, layer: i32) -> vec4<f32> {
    let known = select(1.0, 0.0, layer > MOUNTAIN);
    let l = select(min(layer, 9), 3, layer == MOUNTAIN);
    let uv = xz / REPEAT;
    let a = textureSample(ground_layers, repeat_sampler, uv, l).rgb;
    let b = textureSample(ground_layers, repeat_sampler, mat2x2<f32>(0.8, -0.6, 0.6, 0.8) * uv * 0.43 + 0.37, l).rgb;
    let paint = mix(a, b, 0.3 * fbm(xz * 0.21));
    // Lift the brushwork that mipmapping flattens at gameplay distance.
    let mean = textureSampleLevel(ground_layers, repeat_sampler, uv, l, 9.0).rgb;
    return vec4<f32>(max(mean + (paint - mean) * 1.8, vec3<f32>(0.0)) * known, known);
}

struct Paint {
    color: vec4<f32>,
    // 1 inside river and mountain cells, blended across their ragged edges.
    river: f32,
    rock: f32,
};

fn ground_paint(xz: vec2<f32>) -> Paint {
    // Blend between simulation cells, which are finer than world units.
    let cell = xz / g.map_size * vec2<f32>(textureDimensions(ground_index));
    let gp = cell - 0.5;
    let base = floor(gp);
    var f = fract(gp) + (vec2<f32>(fbm(cell * 1.7), fbm(cell * 1.7 + 11.3)) - 0.5) * 0.7;
    f = smoothstep(vec2<f32>(0.3), vec2<f32>(0.7), clamp(f, vec2<f32>(0.0), vec2<f32>(1.0)));
    let l00 = layer_at(base);
    let l10 = layer_at(base + vec2<f32>(1.0, 0.0));
    let l01 = layer_at(base + vec2<f32>(0.0, 1.0));
    let l11 = layer_at(base + vec2<f32>(1.0, 1.0));
    let c = mix(
        mix(paint_sample(xz, l00), paint_sample(xz, l10), f.x),
        mix(paint_sample(xz, l01), paint_sample(xz, l11), f.x),
        f.y,
    );
    let layers = vec4<i32>(l00, l10, l01, l11);
    let r = vec4<f32>(layers == vec4<i32>(RIVER));
    let m = vec4<f32>(layers == vec4<i32>(MOUNTAIN));
    return Paint(
        c,
        mix(mix(r.x, r.y, f.x), mix(r.z, r.w, f.x), f.y),
        mix(mix(m.x, m.y, f.x), mix(m.z, m.w, f.x), f.y),
    );
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
    let paint = ground_paint(xz);
    let painted = paint.color * (1.0 - wild);
    land = land * (1.0 - painted.a) + painted.rgb * in.shade * mix(0.94, 1.06, patch_tone);
    let h = in.world.y;
    // Peaks and steep hillsides are bare rock; the highest summits carry snow.
    let n = normalize(in.normal);
    let steep = max(
        smoothstep(0.22, 0.45, 1.0 - n.y) * smoothstep(0.3, 0.7, h),
        smoothstep(0.3, 0.7, paint.rock) * mix(0.75, 1.0, fbm(xz * 0.8)),
    );
    let rock = srgb_to_linear(vec3<f32>(0.62, 0.58, 0.53)) * mix(0.85, 1.1, fbm(xz * 1.3)) * in.shade;
    land = mix(land, rock, steep * painted.a);
    let snow = smoothstep(1.55, 1.8, h + (fbm(xz * 0.9) - 0.5) * 0.3) * (1.0 - steep * 0.6);
    land = mix(land, vec3<f32>(0.93, 0.95, 0.98), snow * painted.a);
    // Rivers: clear blue-teal water over the painted shallows, with drifting glints.
    let flow = fbm(xz * 2.2 + vec2<f32>(g.time * 0.35, g.time * 0.2));
    let water = srgb_to_linear(vec3<f32>(0.2, 0.58, 0.72)) * mix(0.9, 1.15, flow);
    let glint = smoothstep(0.68, 0.8, flow) * 0.35;
    land = mix(land, mix(water, painted.rgb, 0.18) + vec3<f32>(glint), smoothstep(0.35, 0.65, paint.river) * painted.a);
    let beach = paint_sample(xz, BEACH);
    let color0 = mix(beach.rgb, land, smoothstep(-0.2, -0.08, h));
    var color = mix(color0 * vec3<f32>(0.35, 0.8, 0.85), color0, smoothstep(-0.57, -0.32, h));
    let foam = smoothstep(0.03, 0.0, abs(h + 0.32 - 0.012 * sin(g.time * 1.4 + xz.x * 2.0 + xz.y)));
    color = mix(color, vec3<f32>(1.0), foam * 0.85);
    // The paving, rather than a distorted building image, describes the exact
    // rectangular claim. World UVs join seamlessly between neighboring plots.
    let plot_q = xz - g.placement.xy;
    let preview_plot = g.placement_color.a > 0.0
        && all(plot_q >= vec2<f32>(0.0)) && all(plot_q < g.placement.zw);
    let surface = i32(round(textureLoad(ground_index, vec2<i32>(floor(xz * 2.0)), 0).g * 255.0));
    let occupied_plot = surface == 255;
    // Mirror the illustrated swatch at its edges: neighboring repeats meet at
    // identical texels, without blurring or doubling the hand-drawn outlines.
    let paving_uv = 1.0 - abs(fract(xz / 1.8) * 2.0 - 1.0);
    // WebGL requires one sampler per texture. Clamp inside the selected mip
    // manually, retaining minification filtering without wrapping edge texels.
    let texels = vec2<f32>(textureDimensions(ground_layers));
    let density = max(length(dpdx(xz / 0.9) * texels), length(dpdy(xz / 0.9) * texels));
    let mip = clamp(ceil(log2(max(density, 1.0))), 0.0, 9.0);
    let inset = vec2<f32>(0.5 * exp2(mip)) / texels;
    let paving = textureSampleLevel(ground_layers, repeat_sampler, clamp(paving_uv, inset, 1.0 - inset), 10, mip).rgb;
    // Roads follow the terrain, using the existing painted earth and paving.
    let earth = paint_sample(xz, 7).rgb * vec3<f32>(0.75, 0.78, 0.83);
    if surface == 1 { color = earth; }
    if surface == 2 { color = paving; }
    if surface == 3 { color = mix(color, earth, 0.55); }
    if occupied_plot || preview_plot {
        color = paving;
    }
    color *= cel_light(n);
    color = world_light(color, xz);
    // Square simulation cells project as diamonds. Paint their lines on the
    // terrain itself so the grid follows hills rather than floating above them.
    let cell = xz * 2.0;
    let pixel = max(fwidth(cell), vec2<f32>(0.0001));
    let distance_to_line = min(fract(cell), 1.0 - fract(cell));
    let lines = 1.0 - smoothstep(vec2<f32>(0.0), pixel * 1.2, distance_to_line);
    let line = max(lines.x, lines.y) * (1.0 - smoothstep(0.2, 0.6, max(pixel.x, pixel.y)));
    let q = xz - g.placement.xy;
    let edge = min(q, g.placement.zw - q);
    let near = all(q >= vec2<f32>(-2.0)) && all(q <= g.placement.zw + 2.0);
    let placing = g.placement_color.a > 0.0;
    let known = layer_at(floor(cell)) <= MOUNTAIN && wild < 0.01;
    if known && (g.grid.x > 0.5 || (placing && near)) {
        color = mix(color, vec3<f32>(0.88, 0.92, 0.82), line * 0.4);
    }
    if known && placing && all(edge >= vec2<f32>(0.0)) {
        let boundary = 1.0 - smoothstep(0.0, max(pixel.x, pixel.y), min(edge.x, edge.y) * 2.0);
        color = mix(color, g.placement_color.rgb, 0.18 + boundary * 0.65);
    }
    return vec4<f32>(distance_fog(color, in.world), 1.0);
}
