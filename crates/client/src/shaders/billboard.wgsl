// Generated illustrated sprites (villagers, trees, resources, buildings) as
// camera-facing quads anchored on the ground. Depth is alpha-tested to the
// painted figure so the ink pass outlines it.
//
// A flat quad stands at its anchor, but the picture shows a solid object whose
// front (a temple's steps, a villager's feet) reaches toward the viewer. So the
// whole sprite takes the depth of a point pulled toward the camera (`pull`): rising ground just in front of the anchor no longer cuts it off,
// while a real ridge between it and the camera still hides it.
@group(1) @binding(0) var sheet: texture_2d<f32>;
@group(1) @binding(1) var sheet_sampler: sampler;

struct Instance {
    @location(0) anchor: vec3<f32>,
    @location(1) size: vec2<f32>,
    @location(2) pivot: vec2<f32>,
    @location(3) uv: vec4<f32>,
    @location(4) pull: f32,
    @location(5) tint: vec4<f32>,
    @location(6) footprint: vec2<f32>,
};

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) anchor: vec3<f32>,
    @location(2) tint: vec4<f32>,
    // Screen-right distance from the anchor, for building depth.
    @location(3) across: f32,
    @location(4) @interpolate(flat) footprint: vec2<f32>,
    @location(5) @interpolate(flat) pull: f32,
};

struct FOut {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

// A building's depth in this screen column: the footprint's front edge below
// it. The anchor is the footprint corner nearest the camera; from there one
// front edge runs along -x (the width) and the other along -z (the depth).
fn building_depth(in: VOut) -> f32 {
    let right = g.camera_right.xyz;
    let along_x = dot(vec3<f32>(-1.0, 0.0, 0.0), right);
    let along_z = dot(vec3<f32>(0.0, 0.0, -1.0), right);
    var point = bend(in.anchor);
    if in.across * along_x >= 0.0 {
        point.x -= clamp(in.across / along_x, 0.0, in.footprint.x);
    } else {
        point.z -= clamp(in.across / along_z, 0.0, in.footprint.y);
    }
    let toward_camera = normalize(cross(g.camera_right.xyz, g.camera_up.xyz));
    let clip = g.view_proj * vec4<f32>(point + toward_camera * in.pull, 1.0);
    return max(0.0, clip.z / clip.w);
}

@vertex
fn vs(@builtin(vertex_index) index: u32, inst: Instance) -> VOut {
    let corners = array<vec2<f32>, 6>(vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0));
    let q = corners[index];
    let anchor = bend(inst.anchor);
    let local = q - inst.pivot;
    let offset = local.x * inst.size.x * g.camera_right.xyz + local.y * inst.size.y * g.camera_up.xyz;
    var out: VOut;
    out.clip = g.view_proj * vec4<f32>(anchor + offset, 1.0);
    let toward_camera = normalize(cross(g.camera_right.xyz, g.camera_up.xyz));
    let front = g.view_proj * vec4<f32>(anchor + toward_camera * inst.pull, 1.0);
    // At close zoom the ground anchor can pass the near plane while the
    // roof still overlaps the viewport. Keep the quad for rasterization;
    // its screen bounds, alpha and per-column depth decide what is drawn.
    out.clip.z = max(0.0, front.z / front.w) * out.clip.w;
    out.uv = vec2<f32>(mix(inst.uv.x, inst.uv.z, q.x), mix(inst.uv.w, inst.uv.y, q.y));
    out.anchor = inst.anchor;
    out.tint = inst.tint;
    out.across = dot(offset, g.camera_right.xyz);
    out.footprint = inst.footprint;
    out.pull = inst.pull;
    return out;
}

@fragment
fn fs(in: VOut) -> FOut {
    let texel = textureSample(sheet, sheet_sampler, in.uv);
    if texel.a < 0.5 {
        discard;
    }
    let color = world_light(texel.rgb, in.anchor.xz);
    var out: FOut;
    // Opaque artwork skips grading. Ghost alpha remains its blend opacity.
    out.color = vec4<f32>(distance_fog(color, in.anchor) * in.tint.rgb, select(0.0, in.tint.a, in.tint.a < 1.0));
    out.depth = select(in.clip.z, building_depth(in), in.footprint.x > 0.0);
    return out;
}

// A villager hidden behind a building (or anything nearer) shows through as a
// flat team-blue shape, as in Age of Empires II. Drawn after every sprite with
// a depth test of "greater", so only the covered parts appear.
@fragment
fn fs_silhouette(in: VOut) -> @location(0) vec4<f32> {
    let texel = textureSample(sheet, sheet_sampler, in.uv);
    if texel.a < 0.5 {
        discard;
    }
    return vec4<f32>(0.16, 0.36, 0.78, 0.7);
}

// Cast shadows on the ground, as in Age of Empires II: the painted figure
// itself is the shadow, with the sun high at the screen's top left. A unit's
// painted points slide down-right on the ground by a short fraction of their
// painted height, draped over the drawn terrain. A building's picture is laid
// on its plot and nudged down-right, so the shadow peeks out along its right
// and bottom edges; the building covers the rest. The pass writes depth at
// the ground, so overlapping shadows on one level fail the depth test
// instead of darkening twice.
struct ShadowOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world: vec3<f32>,
    @location(2) @interpolate(flat) strength: f32,
};

// Unit ground offset per unit of painted height: screen-right and toward the camera.
const SHADOW_RIGHT: f32 = 0.55;
const SHADOW_DOWN: f32 = 0.4;
// Building nudge, as a fraction of the sprite's width (screen-right) and
// height (screen-down).
const DROP_RIGHT: f32 = 0.05;
const DROP_DOWN: f32 = 0.04;

@vertex
fn vs_shadow(@builtin(vertex_index) index: u32, inst: Instance) -> ShadowOut {
    let corners = array<vec2<f32>, 6>(vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0));
    let q = corners[index];
    let building = inst.footprint.x > 0.0;
    let local = q - inst.pivot;
    let right = g.camera_right.xyz;
    let toward_camera = normalize(cross(right, g.camera_up.xyz));
    let down = normalize(vec3<f32>(toward_camera.x, 0.0, toward_camera.z));
    var world: vec3<f32>;
    if inst.tint.a < 1.0 {
        // Placement ghosts cast nothing.
        world = inst.anchor;
    } else if building {
        // Screen-up on a flat plot is ground away from the camera, foreshortened.
        let foreshorten = length(toward_camera.xz);
        world = inst.anchor + (local.x * inst.size.x + DROP_RIGHT * inst.size.x) * right
            - ((local.y - DROP_DOWN) * inst.size.y / foreshorten) * down;
    } else {
        world = inst.anchor + local.x * inst.size.x * right
            + local.y * inst.size.y * (SHADOW_RIGHT * right + SHADOW_DOWN * down);
        // Drape on the drawn ground.
        world.y = ground_height(world.xz);
    }
    // Lifted just clear of the ground's depth (under the ink pass threshold)
    // so the shadow paints over it without outlines.
    let away = distance(inst.anchor, g.camera_pos.xyz);
    let lift = 0.01 + 0.0008 * away;
    var out: ShadowOut;
    out.clip = g.view_proj * vec4<f32>(bend(world) + toward_camera * lift, 1.0);
    out.uv = vec2<f32>(mix(inst.uv.x, inst.uv.z, q.x), mix(inst.uv.w, inst.uv.y, q.y));
    out.world = world;
    out.strength = 1.0 - smoothstep(45.0, 90.0, away);
    return out;
}

@fragment
fn fs_shadow(in: ShadowOut) -> @location(0) vec4<f32> {
    if textureSample(sheet, sheet_sampler, in.uv).a < 0.5 {
        discard;
    }
    let strength = in.strength * smoothstep(0.55, 0.95, visibility(in.world.xz));
    // Multiplied onto the lit ground: a cool, soft darkening.
    return vec4<f32>(mix(vec3<f32>(1.0), vec3<f32>(0.5, 0.56, 0.7), strength * 0.9), 1.0);
}
