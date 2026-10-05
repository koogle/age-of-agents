// Deep blue-teal sea with gentle swell and a soft sun glint.
struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
};

@vertex
fn vs(@location(0) position: vec3<f32>) -> VOut {
    var p = position;
    p.x += g.curve_center.x;
    p.z += g.curve_center.y;
    p.y += sin(p.x * 0.9 + g.time * 1.1) * 0.025 + cos(p.z * 0.7 + g.time * 0.8) * 0.025;
    var out: VOut;
    out.world = p;
    out.clip = g.view_proj * vec4<f32>(bend(p), 1.0);
    return out;
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let n = normalize(vec3<f32>(-0.025 * 0.9 * cos(in.world.x * 0.9 + g.time * 1.1), 1.0, 0.025 * 0.7 * sin(in.world.z * 0.7 + g.time * 0.8)));
    let base = srgb_to_linear(vec3<f32>(0.173, 0.553, 0.698));
    let view = normalize(g.camera_pos.xyz - in.world);
    let half_dir = normalize(view + g.sun_dir.xyz);
    let spec = pow(max(dot(n, half_dir), 0.0), 70.0) * 0.6;
    var color = base * cel_light(n) + vec3<f32>(spec);
    color = distance_fog(color, in.world);
    return vec4<f32>(color, 0.9);
}
