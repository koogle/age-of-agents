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
    return clip.z / clip.w;
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
    out.clip.z = front.z / front.w * out.clip.w;
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
