// Flat ground marks: soft blob shadows under sprites and selection rings.
struct Instance {
    @location(0) center: vec3<f32>,
    @location(1) radius: f32,
    @location(2) color: vec4<f32>,
    @location(3) ring: f32,
};

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) ring: f32,
};

@vertex
fn vs(@builtin(vertex_index) index: u32, inst: Instance) -> VOut {
    let corners = array<vec2<f32>, 6>(vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0), vec2(-1.0, 1.0), vec2(1.0, -1.0), vec2(1.0, 1.0));
    let q = corners[index];
    let world = inst.center + vec3<f32>(q.x * inst.radius, 0.0, q.y * inst.radius);
    var out: VOut;
    out.clip = g.view_proj * vec4<f32>(bend(world), 1.0);
    out.local = q;
    out.color = inst.color;
    out.ring = inst.ring;
    return out;
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let r = length(in.local);
    var alpha: f32;
    if in.ring > 0.5 {
        alpha = smoothstep(0.72, 0.8, r) * (1.0 - smoothstep(0.92, 1.0, r));
    } else {
        alpha = 1.0 - smoothstep(0.55, 1.0, r);
    }
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
