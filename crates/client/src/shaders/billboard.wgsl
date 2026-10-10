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
    @location(7) base: vec2<f32>,
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

// Cast shadows on the ground under a sun high to the screen's left. A unit's painted
// card is a real occluder: every point is projected along the sun onto the
// ground and draped over the drawn terrain. A building is its footprint box,
// swept along the sun and draped over the terrain. Each instance draws three
// quads; units collapse the two they do not use. The pass writes depth at the ground, so overlapping
// shadows on one level fail the depth test instead of darkening twice.
struct ShadowOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world: vec3<f32>,
    @location(2) @interpolate(flat) strength: f32,
    @location(3) @interpolate(flat) textured: f32,
};

// The sun stands high to the screen's left, only a little in front of the
// scene (the painted sprites are lit from the front left). Shadows then run
// nearly horizontally to the right on screen, which reads as lying flat on
// the ground; a sun further in front would sweep them up along the walls.
const SUN_UP: f32 = 0.9;
const SUN_LEFT: f32 = 1.0;
const SUN_FRONT: f32 = 0.15;
const SHADOW_GRID: u32 = 4u;

// Ground offset of a point `height` above the plot under the sun.
fn shadow_vector(sun: vec3<f32>, height: f32) -> vec3<f32> {
    return vec3<f32>(-sun.x / sun.y, 0.0, -sun.z / sun.y) * height;
}

@vertex
fn vs_shadow(@builtin(vertex_index) index: u32, inst: Instance) -> ShadowOut {
    let corners = array<vec2<f32>, 6>(vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0));
    // Buildings subdivide each quad into a SHADOW_GRID² grid so the swept
    // box can drape over the terrain; units draw the first six vertices.
    let per_quad = SHADOW_GRID * SHADOW_GRID * 6u;
    let quad = index / per_quad;
    let tile = (index % per_quad) / 6u;
    var q = corners[index % 6u];
    if inst.footprint.x > 0.0 {
        q = (vec2<f32>(f32(tile % SHADOW_GRID), f32(tile / SHADOW_GRID)) + q) / f32(SHADOW_GRID);
    }
    let local = q - inst.pivot;
    let right = g.camera_right.xyz;
    let toward_camera = normalize(cross(right, g.camera_up.xyz));
    let sun = normalize(SUN_FRONT * toward_camera + vec3<f32>(0.0, SUN_UP, 0.0) - SUN_LEFT * right);
    // Buildings with a painted base cast their box; units and tall towers
    // cast their painted card.
    let building = inst.footprint.x > 0.0 && inst.base.x > 0.0;
    var world = inst.anchor;
    var uv = vec2<f32>(0.0);
    if inst.tint.a < 1.0 || (quad > 0u && !building) {
        // Placement ghosts cast nothing; cards use the first quad only.
        world = inst.anchor;
    } else if building {
        // A building is a box on its plot, not a card: the shadow is its
        // footprint swept along the sun. The top face lands at the shadow
        // vector; the two trailing edges sweep toward it. Box height comes
        // from the art: the image top above the anchor, less the far
        // footprint corner's screen rise, read back through the camera pitch.
        let rise = (1.0 - inst.pivot.y) * inst.size.y;
        let far = -dot(vec3<f32>(inst.footprint.x, 0.0, inst.footprint.y), g.camera_up.xyz);
        let height = clamp((rise - far) / g.camera_up.y * 0.8, 0.0, 6.0);
        let v = shadow_vector(sun, height);
        // The painted base, centred on the plot, is what casts.
        let w = inst.base.x;
        let d = inst.base.y;
        let corner = inst.anchor - vec3<f32>(inst.footprint.x - w, 0.0, inst.footprint.y - d) * 0.5;
        if quad == 0u {
            world = corner + vec3<f32>(-q.x * w, 0.0, -q.y * d) + v;
        } else if quad == 1u {
            let x = select(corner.x, corner.x - w, v.x > 0.0);
            world = vec3<f32>(x, corner.y, corner.z - q.x * d) + q.y * v;
        } else {
            let z = select(corner.z, corner.z - d, v.z > 0.0);
            world = vec3<f32>(corner.x - q.x * w, corner.y, z) + q.y * v;
        }
        world.y = ground_height(world.xz);
    } else {
        // The card as drawn, slid along the sun down to the ground, then
        // draped on the drawn terrain. A tower's anchor is its plot's front
        // corner, so heights are measured from the footprint centre, where
        // the painted base stands.
        let centre = inst.anchor - vec3<f32>(inst.footprint.x, 0.0, inst.footprint.y) * 0.5;
        let rise = dot(centre - inst.anchor, g.camera_up.xyz);
        let card = centre + local.x * inst.size.x * right + (local.y * inst.size.y - rise) * g.camera_up.xyz;
        world = card - sun * ((card.y - centre.y) / sun.y);
        world.y = ground_height(world.xz);
        uv = vec2<f32>(mix(inst.uv.x, inst.uv.z, q.x), mix(inst.uv.w, inst.uv.y, q.y));
    }
    // Lifted just clear of the ground's depth (under the ink pass threshold)
    // so the shadow paints over it without outlines.
    let away = distance(inst.anchor, g.camera_pos.xyz);
    let lift = 0.01 + 0.0008 * away;
    var out: ShadowOut;
    out.clip = g.view_proj * vec4<f32>(bend(world) + toward_camera * lift, 1.0);
    out.uv = uv;
    out.world = world;
    out.textured = select(0.0, 1.0, !building);
    out.strength = 1.0 - smoothstep(45.0, 90.0, away);
    return out;
}

@fragment
fn fs_shadow(in: ShadowOut) -> @location(0) vec4<f32> {
    let texel = textureSample(sheet, sheet_sampler, in.uv);
    if in.textured > 0.5 && texel.a < 0.5 {
        discard;
    }
    let strength = in.strength * smoothstep(0.55, 0.95, visibility(in.world.xz));
    // Multiplied onto the lit ground: a cool, soft darkening.
    return vec4<f32>(mix(vec3<f32>(1.0), vec3<f32>(0.5, 0.56, 0.7), strength * 0.9), 1.0);
}
