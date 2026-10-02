// Finish passes: fine one-pixel ink lines where the screen-space Laplacian of
// inverse depth is non-zero (silhouettes and creases), then a tilt-shift blur
// that grows away from a focus band, then ACES tone mapping.
struct Post {
    texel: vec2<f32>,
    near: f32,
    far: f32,
    step: vec2<f32>,
    encode_srgb: f32,
    final_pass: f32,
};

@group(0) @binding(0) var<uniform> p: Post;
@group(0) @binding(1) var image: texture_2d<f32>;
@group(0) @binding(2) var image_sampler: sampler;
// Depth bound as an unfilterable float texture: WebGL2 can texelFetch that.
@group(0) @binding(3) var depth: texture_2d<f32>;

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) index: u32) -> VOut {
    let q = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var out: VOut;
    out.clip = vec4<f32>(q * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(q.x, 1.0 - q.y);
    return out;
}

fn inverse_depth(pixel: vec2<i32>) -> f32 {
    let size = vec2<i32>(textureDimensions(depth));
    let d = textureLoad(depth, clamp(pixel, vec2<i32>(0), size - 1), 0).r;
    let view_z = p.near * p.far / (p.far - d * (p.far - p.near));
    return 1.0 / view_z;
}

fn crease(pixel: vec2<i32>, o: vec2<i32>) -> f32 {
    let c = inverse_depth(pixel);
    return abs(inverse_depth(pixel + o) + inverse_depth(pixel - o) - 2.0 * c) / c;
}

@fragment
fn ink(in: VOut) -> @location(0) vec4<f32> {
    let color = textureSampleLevel(image, image_sampler, in.uv, 0.0);
    let pixel = vec2<i32>(in.clip.xy);
    let edge = max(max(crease(pixel, vec2<i32>(1, 0)), crease(pixel, vec2<i32>(0, 1))),
                   max(crease(pixel, vec2<i32>(1, 1)), crease(pixel, vec2<i32>(1, -1))));
    var amount = smoothstep(0.0025, 0.012, edge) * 0.9;
    amount *= mix(1.0, 0.55, smoothstep(30.0, 90.0, 1.0 / inverse_depth(pixel)));
    return vec4<f32>(mix(color.rgb, color.rgb * vec3<f32>(0.16, 0.11, 0.08), amount), 1.0);
}

fn aces(color_in: vec3<f32>) -> vec3<f32> {
    // three.js ACESFilmicToneMapping (exposure 1).
    let input = mat3x3<f32>(vec3(0.59719, 0.07600, 0.02840), vec3(0.35458, 0.90834, 0.13383), vec3(0.04823, 0.01566, 0.83777));
    let output = mat3x3<f32>(vec3(1.60475, -0.10208, -0.00327), vec3(-0.53108, 1.10813, -0.07276), vec3(-0.07367, -0.00605, 1.07602));
    var c = input * (color_in / 0.6);
    let a = c * (c + 0.0245786) - 0.000090537;
    let b = c * (0.983729 * c + 0.4329510) + 0.238081;
    c = output * (a / b);
    return clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    return select(1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3<f32>(0.0031308));
}

@fragment
fn blur(in: VOut) -> @location(0) vec4<f32> {
    let away = smoothstep(0.6 * 0.35, 0.6, abs(in.uv.y - 0.46));
    let s = p.step * away * 1.1;
    var color = textureSampleLevel(image, image_sampler, in.uv, 0.0).rgb * 0.227;
    color += (textureSampleLevel(image, image_sampler, in.uv + s * 1.385, 0.0).rgb + textureSampleLevel(image, image_sampler, in.uv - s * 1.385, 0.0).rgb) * 0.316;
    color += (textureSampleLevel(image, image_sampler, in.uv + s * 3.231, 0.0).rgb + textureSampleLevel(image, image_sampler, in.uv - s * 3.231, 0.0).rgb) * 0.07;
    if p.final_pass > 0.5 {
        let grey = dot(color, vec3<f32>(0.299, 0.587, 0.114));
        color = mix(vec3<f32>(grey), color, 1.12);
        let edge = smoothstep(0.95, 0.35, length(in.uv - vec2<f32>(0.5, 0.48)));
        color *= mix(vec3<f32>(0.86, 0.84, 0.8), vec3<f32>(1.0), edge);
        color = aces(color);
        if p.encode_srgb > 0.5 {
            color = linear_to_srgb(color);
        }
    }
    return vec4<f32>(color, 1.0);
}
