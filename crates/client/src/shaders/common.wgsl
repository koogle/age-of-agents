// Shared by every world shader: camera, planet curve, noise, fog of war,
// drifting cloud shadows, and the toon light ramp.
struct Globals {
    view_proj: mat4x4<f32>,
    camera_right: vec4<f32>,
    camera_up: vec4<f32>,
    camera_pos: vec4<f32>,
    sun_dir: vec4<f32>,
    map_size: vec2<f32>,
    time: f32,
    curve: f32,
    curve_center: vec2<f32>,
    fog_near: f32,
    fog_far: f32,
};

@group(0) @binding(0) var<uniform> g: Globals;
// rgb = known biome colour (sRGB bytes), a = visibility (0 unseen, 0.5 explored, 1 visible).
@group(0) @binding(1) var cells: texture_2d<f32>;
@group(0) @binding(2) var linear_clamp: sampler;

fn hash2(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash2(i), hash2(i + vec2<f32>(1.0, 0.0)), u.x),
               mix(hash2(i + vec2<f32>(0.0, 1.0)), hash2(i + vec2<f32>(1.0, 1.0)), u.x), u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    return vnoise(p) * 0.55 + vnoise(p * 2.1 + 7.3) * 0.3 + vnoise(p * 4.3 + 1.7) * 0.15;
}

// Far zoom bends the world down away from the camera target into a small planet.
fn bend(world: vec3<f32>) -> vec3<f32> {
    let away = world.xz - g.curve_center;
    return vec3<f32>(world.x, world.y - g.curve * dot(away, away), world.z);
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    return pow(c, vec3<f32>(2.2));
}

fn visibility(xz: vec2<f32>) -> f32 {
    let outside = max(max(-xz, xz - g.map_size), vec2<f32>(0.0));
    let scenery = smoothstep(0.3, 1.8, length(outside));
    let uv = clamp(xz / g.map_size, vec2<f32>(0.0), vec2<f32>(1.0));
    return max(textureSampleLevel(cells, linear_clamp, uv, 0.0).a, scenery);
}

// Cloud shadows, greyed remembered land, and cumulus over unexplored land.
fn world_light(color_in: vec3<f32>, xz: vec2<f32>) -> vec3<f32> {
    var color = color_in;
    let cloud = fbm(xz * 0.07 + vec2<f32>(g.time * 0.018, g.time * 0.011));
    color *= mix(1.0, 0.86, smoothstep(0.52, 0.72, cloud));
    let vis = visibility(xz);
    let seen = smoothstep(0.08, 0.45, vis);
    let lit = smoothstep(0.55, 0.95, vis);
    let grey = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    let remembered = mix(mix(vec3<f32>(grey), color, 0.5) * 0.85, vec3<f32>(0.92, 0.93, 0.95), 0.25);
    color = mix(remembered, color, lit);
    let mist = fbm(xz * 0.28 + vec2<f32>(g.time * 0.03, -g.time * 0.02));
    let billow = fbm(xz * 0.9 - vec2<f32>(g.time * 0.05, g.time * 0.04));
    let unknown = mix(vec3<f32>(0.6, 0.66, 0.74), vec3<f32>(0.98, 0.96, 0.92), smoothstep(0.3, 0.72, mist * 0.65 + billow * 0.35));
    return mix(srgb_to_linear(unknown), color, seen);
}

fn distance_fog(color: vec3<f32>, world: vec3<f32>) -> vec3<f32> {
    let d = distance(world, g.camera_pos.xyz);
    return mix(color, srgb_to_linear(vec3<f32>(0.812, 0.898, 0.949)), smoothstep(g.fog_near, g.fog_far, d));
}

// The soft two-tone toon ramp: half-Lambert through a short painted terminator.
fn toon(ndl: f32) -> f32 {
    let u = ndl * 0.5 + 0.5;
    let x = clamp(u * 8.0 - 0.5, 0.0, 7.0);
    var ramp = array<f32, 8>(0.463, 0.463, 0.486, 0.769, 0.965, 1.0, 1.0, 1.0);
    let i = u32(floor(x));
    let j = min(i + 1u, 7u);
    return mix(ramp[i], ramp[j], fract(x));
}

// Hemisphere sky light plus the warm sun through the toon ramp (three.js scale).
fn cel_light(normal: vec3<f32>) -> vec3<f32> {
    let sky = srgb_to_linear(vec3<f32>(0.624, 0.784, 1.0));
    let ground = srgb_to_linear(vec3<f32>(0.549, 0.498, 0.416));
    let hemi = mix(ground, sky, normal.y * 0.5 + 0.5) * 1.25;
    let sun = srgb_to_linear(vec3<f32>(1.0, 0.941, 0.831)) * 3.0 * toon(dot(normal, g.sun_dir.xyz));
    return (hemi + sun) / 3.14159265;
}
