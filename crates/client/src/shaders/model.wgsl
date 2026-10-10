// Generated 3D building models: textured meshes placed on their plots, lit
// with the shared toon ramp and fog of war. A second entry flattens the same
// mesh along the shadow sun onto the plot plane for its cast shadow.
@group(1) @binding(0) var albedo: texture_2d<f32>;
@group(1) @binding(1) var albedo_sampler: sampler;

struct VIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) translation: vec3<f32>,
    @location(4) scale: f32,
};

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vs(in: VIn) -> VOut {
    var out: VOut;
    out.world = in.translation + in.position * in.scale;
    out.normal = in.normal;
    out.uv = in.uv;
    out.clip = g.view_proj * vec4<f32>(bend(out.world), 1.0);
    return out;
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let texel = textureSample(albedo, albedo_sampler, in.uv);
    // Generated albedos are muted next to the painted sprites; lift them.
    var color = srgb_to_linear(min(texel.rgb * 1.45, vec3<f32>(1.0))) * cel_light(normalize(in.normal));
    color = world_light(color, in.world.xz);
    return vec4<f32>(distance_fog(color, in.world), 0.0);
}

@vertex
fn vs_shadow(in: VIn) -> VOut {
    let point = in.translation + in.position * in.scale;
    let sun = shadow_sun();
    var world = point - sun * ((point.y - in.translation.y) / sun.y);
    let right = g.camera_right.xyz;
    let toward_camera = normalize(cross(right, g.camera_up.xyz));
    let lift = 0.01 + 0.0008 * distance(in.translation, g.camera_pos.xyz);
    var out: VOut;
    out.world = world;
    out.normal = in.normal;
    out.uv = in.uv;
    out.clip = g.view_proj * vec4<f32>(bend(world) + toward_camera * lift, 1.0);
    return out;
}

@fragment
fn fs_shadow(in: VOut) -> @location(0) vec4<f32> {
    let away = distance(in.world, g.camera_pos.xyz);
    let strength = (1.0 - smoothstep(45.0, 90.0, away)) * smoothstep(0.55, 0.95, visibility(in.world.xz));
    return vec4<f32>(mix(vec3<f32>(1.0), vec3<f32>(0.5, 0.56, 0.7), strength * 0.9), 1.0);
}
