// The painted interface: textured quads from one atlas (generated coins,
// icons and Nunito glyphs), soft rounded glass pills, rings, and the globe
// minimap read straight from the fog-of-war cell texture.
struct Hud {
    screen: vec2<f32>,
    encode_srgb: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> hud: Hud;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var atlas_sampler: sampler;
@group(0) @binding(3) var cells: texture_2d<f32>;

struct Quad {
    @location(0) rect: vec4<f32>,
    @location(1) uv: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(3) params: vec4<f32>,
};

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) params: vec4<f32>,
    @location(4) size: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) index: u32, quad: Quad) -> VOut {
    let corners = array<vec2<f32>, 6>(vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0));
    let q = corners[index];
    let pixel = quad.rect.xy + q * quad.rect.zw;
    var out: VOut;
    out.clip = vec4<f32>(pixel.x / hud.screen.x * 2.0 - 1.0, 1.0 - pixel.y / hud.screen.y * 2.0, 0.0, 1.0);
    out.local = q;
    out.uv = mix(quad.uv.xy, quad.uv.zw, q);
    out.color = quad.color;
    out.params = quad.params;
    out.size = quad.rect.zw;
    return out;
}

fn to_linear(c: vec3<f32>) -> vec3<f32> {
    return pow(c, vec3<f32>(2.2));
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let mode = i32(in.params.x);
    var color = vec4<f32>(to_linear(in.color.rgb), in.color.a);
    let p = (in.local - 0.5) * in.size;
    let texel = textureSample(atlas, atlas_sampler, in.uv);
    if mode == 0 {
        color = vec4<f32>(texel.rgb * color.rgb, texel.a * color.a);
    } else if mode == 1 {
        // Rounded rectangle with a soft one-pixel edge.
        let r = in.params.y;
        let d = length(max(abs(p) - (in.size * 0.5 - r), vec2<f32>(0.0))) - r;
        color.a *= clamp(0.5 - d, 0.0, 1.0);
    } else if mode == 2 {
        // Ring of width params.y.
        let d = abs(length(p) - (in.size.x * 0.5 - in.params.y * 0.5)) - in.params.y * 0.5;
        color.a *= clamp(0.5 - d, 0.0, 1.0);
    } else {
        // Globe minimap: known biome colours, unexplored as parchment.
        let d = length(p) - in.size.x * 0.5;
        let cell = textureSampleLevel(cells, atlas_sampler, in.uv, 0.0);
        let parchment = vec3<f32>(0.93, 0.88, 0.76);
        let land = mix(parchment, cell.rgb, smoothstep(0.05, 0.4, cell.a));
        let inside = all(in.uv >= vec2<f32>(0.0)) && all(in.uv <= vec2<f32>(1.0));
        let sea = vec3<f32>(0.55, 0.78, 0.86);
        color = vec4<f32>(to_linear(select(sea, land, inside)), clamp(0.5 - d, 0.0, 1.0));
    }
    if hud.encode_srgb > 0.5 {
        color = vec4<f32>(pow(color.rgb, vec3<f32>(1.0 / 2.2)), color.a);
    }
    return color;
}
