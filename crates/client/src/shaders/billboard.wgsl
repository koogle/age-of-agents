// Generated illustrated sprites (villagers, trees, resources, buildings) as
// camera-facing quads anchored on the ground. Depth is alpha-tested to the
// painted figure so the ink pass outlines it.
@group(1) @binding(0) var sheet: texture_2d<f32>;
@group(1) @binding(1) var sheet_sampler: sampler;

struct Instance {
    @location(0) anchor: vec3<f32>,
    @location(1) size: vec2<f32>,
    @location(2) pivot: vec2<f32>,
    @location(3) uv: vec4<f32>,
};

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) anchor: vec3<f32>,
};

@vertex
fn vs(@builtin(vertex_index) index: u32, inst: Instance) -> VOut {
    let corners = array<vec2<f32>, 6>(vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0));
    let q = corners[index];
    let anchor = bend(inst.anchor);
    let offset = (q.x - inst.pivot.x) * inst.size.x * g.camera_right.xyz + (q.y - inst.pivot.y) * inst.size.y * g.camera_up.xyz;
    var out: VOut;
    out.clip = g.view_proj * vec4<f32>(anchor + offset, 1.0);
    out.uv = vec2<f32>(mix(inst.uv.x, inst.uv.z, q.x), mix(inst.uv.w, inst.uv.y, q.y));
    out.anchor = inst.anchor;
    return out;
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let texel = textureSample(sheet, sheet_sampler, in.uv);
    if texel.a < 0.5 {
        discard;
    }
    let color = world_light(texel.rgb, in.anchor.xz);
    return vec4<f32>(distance_fog(color, in.anchor), 1.0);
}
