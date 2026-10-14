// Generated ground shadows. Each mask was authored in screen space around its
// sprite, so its quad stands where the sprite's would; every corner then slides
// along the view ray onto the ground plane at the anchor. In an orthographic
// view that keeps the screen position and gives the ground's depth, so terrain
// in front still hides the shadow while the ground behind receives it.
@group(1) @binding(0) var sheet: texture_2d<f32>;
@group(1) @binding(1) var sheet_sampler: sampler;

struct Instance {
    @location(0) anchor: vec3<f32>,
    @location(1) size: vec2<f32>,
    @location(2) pivot: vec2<f32>,
    @location(3) uv: vec4<f32>,
    @location(4) pull: f32,
    // The atlas channel holding this mask.
    @location(5) channel: vec4<f32>,
    @location(6) footprint: vec2<f32>,
};

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) channel: vec4<f32>,
    @location(2) ground: vec3<f32>,
};

@vertex
fn vs(@builtin(vertex_index) index: u32, inst: Instance) -> VOut {
    let corners = array<vec2<f32>, 6>(vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0));
    let q = corners[index];
    let local = q - inst.pivot;
    let upright = inst.anchor + local.x * inst.size.x * g.camera_right.xyz + local.y * inst.size.y * g.camera_up.xyz;
    let toward_camera = normalize(cross(g.camera_right.xyz, g.camera_up.xyz));
    let ground = upright + toward_camera * ((inst.anchor.y - upright.y) / toward_camera.y);
    var out: VOut;
    // The lift keeps the mask over small bumps and level plots; real hills still hide it.
    out.clip = g.view_proj * vec4<f32>(bend(ground) + toward_camera * 0.4, 1.0);
    out.uv = vec2<f32>(mix(inst.uv.x, inst.uv.z, q.x), mix(inst.uv.w, inst.uv.y, q.y));
    out.channel = inst.channel;
    out.ground = ground;
    return out;
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let coverage = dot(textureSample(sheet, sheet_sampler, in.uv), in.channel);
    let strength = 0.55 * coverage * visibility(in.ground.xz);
    if strength < 0.004 {
        discard;
    }
    // A cool shade over the lit ground; scene alpha (grading weight) is untouched.
    // Premultiplied for the OVER blend.
    return vec4<f32>(srgb_to_linear(vec3<f32>(0.16, 0.18, 0.27)) * strength, strength);
}
