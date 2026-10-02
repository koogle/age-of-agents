// Screen-space sky gradient behind everything.
struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) v: f32,
};

@vertex
fn vs(@builtin(vertex_index) index: u32) -> VOut {
    let p = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u)) * 2.0 - 1.0;
    var out: VOut;
    out.clip = vec4<f32>(p, 1.0, 1.0);
    out.v = 0.5 - p.y * 0.5;
    return out;
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let top = srgb_to_linear(vec3<f32>(0.227, 0.541, 0.831));
    let upper = srgb_to_linear(vec3<f32>(0.475, 0.741, 0.914));
    let low = srgb_to_linear(vec3<f32>(0.812, 0.91, 0.957));
    let horizon = srgb_to_linear(vec3<f32>(0.957, 0.925, 0.839));
    var c = mix(top, upper, smoothstep(0.0, 0.4, in.v));
    c = mix(c, low, smoothstep(0.4, 0.75, in.v));
    c = mix(c, horizon, smoothstep(0.75, 1.0, in.v));
    return vec4<f32>(c, 1.0);
}
