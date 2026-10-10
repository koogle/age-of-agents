//! The wgpu renderer: sky, painted cel-shaded ground, sea, ground decals and
//! generated sprite billboards into an offscreen target, then the ink-line,
//! tilt-shift and tone-mapping finish passes onto the window surface.
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::assets::{Assets, Rgba};
use crate::gpu::{
    DEPTH_FORMAT, Gpu, PipelineSpec, SCENE_FORMAT, bind_group, data_texture, ensure_capacity,
    instance_buffer, pipeline, render_target, sampler_entry, shader, texture_entry, uniform_entry,
    upload_texture,
};
use crate::hud::Quad;
use crate::terrain::{self, GroundVertex};

mod map;

const GROUND_SIZE: u32 = 512;
/// Subdivisions per side of each building shadow quad (matches the shader).
const SHADOW_GRID: u32 = 4;

fn globals_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    globals: &wgpu::Buffer,
    cells: &wgpu::Texture,
    linear: &wgpu::Sampler,
    heights: &wgpu::Texture,
) -> wgpu::BindGroup {
    bind_group(
        device,
        "globals",
        layout,
        &[
            globals.as_entire_binding(),
            wgpu::BindingResource::TextureView(&cells.create_view(&Default::default())),
            wgpu::BindingResource::Sampler(linear),
            wgpu::BindingResource::TextureView(&heights.create_view(&Default::default())),
        ],
    )
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct Globals {
    pub view_proj: [[f32; 4]; 4],
    pub camera_right: [f32; 4],
    pub camera_up: [f32; 4],
    pub camera_pos: [f32; 4],
    pub sun_dir: [f32; 4],
    pub map_size: [f32; 2],
    pub time: f32,
    pub curve: f32,
    pub curve_center: [f32; 2],
    pub fog_near: f32,
    pub fog_far: f32,
    pub placement: [f32; 4],
    pub placement_color: [f32; 4],
    pub grid: [f32; 4],
    /// World x/z bounds of the visible ground and its height texture.
    pub ground: [f32; 4],
}

/// One painted sprite standing on the ground.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Sprite {
    pub anchor: [f32; 3],
    /// World width and height of the whole sheet cell.
    pub size: [f32; 2],
    /// Anchor inside the cell, from the bottom-left, in [0, 1].
    pub pivot: [f32; 2],
    /// u0, v0 (top), u1, v1 (bottom); u0 > u1 mirrors.
    pub uv: [f32; 4],
    /// How far toward the camera (world units) the sprite takes its depth
    /// from, so the solid thing it pictures is not cut by the ground in front.
    pub pull: f32,
    /// White/opaque for world sprites; tinted/translucent for placement ghosts.
    pub tint: [f32; 4],
    /// For buildings: the footprint's world width (x) and depth (z), with the
    /// anchor at its corner nearest the camera. Each screen column then takes
    /// the depth of the footprint's front edge below it, so villagers in front
    /// of a wall draw over it and those behind it are hidden (and silhouetted).
    /// Zero for everything else.
    pub footprint: [f32; 2],
    /// For buildings: the painted base's world width and depth, centred on
    /// the footprint, which casts the shadow.
    pub base: [f32; 2],
}

/// A flat mark on the ground: a soft shadow or a selection ring.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Decal {
    pub center: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
    /// 0: shadow, 1: pointer ring, 2: building outline, 3: selected unit ring.
    pub ring: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PostUniform {
    texel: [f32; 2],
    near: f32,
    far: f32,
    step: [f32; 2],
    encode_srgb: f32,
    final_pass: f32,
}

struct Sheet {
    bind_group: wgpu::BindGroup,
}

struct Targets {
    depth: wgpu::TextureView,
    scene: wgpu::TextureView,
    inked: wgpu::TextureView,
    half: wgpu::TextureView,
    ink_group: wgpu::BindGroup,
    blur_h_group: wgpu::BindGroup,
    blur_v_group: wgpu::BindGroup,
}

pub struct Renderer {
    globals: wgpu::Buffer,
    globals_group: wgpu::BindGroup,
    cells: wgpu::Texture,
    /// Ground heights of the visible region, one texel per mesh vertex, so
    /// cast shadows can drape over the terrain.
    heights: wgpu::Texture,
    globals_layout: wgpu::BindGroupLayout,
    terrain_layout: wgpu::BindGroupLayout,
    hud_layout: wgpu::BindGroupLayout,
    linear: wgpu::Sampler,
    repeat: wgpu::Sampler,
    sprite_sampler: wgpu::Sampler,
    ground_layers: wgpu::Texture,
    atlas: wgpu::Texture,
    pub ground_bounds: [f32; 5],
    ground_index: wgpu::Texture,
    terrain_pipeline: wgpu::RenderPipeline,
    terrain_group: wgpu::BindGroup,
    ground: (wgpu::Buffer, wgpu::Buffer, u32),
    sea_pipeline: wgpu::RenderPipeline,
    sea: (wgpu::Buffer, wgpu::Buffer, u32),
    sky_pipeline: wgpu::RenderPipeline,
    sprite_pipeline: wgpu::RenderPipeline,
    /// Villagers hidden behind a building (or anything nearer) show through as
    /// flat team-blue shapes, as in Age of Empires II: depth test greater, no
    /// depth write, drawn after every sprite.
    silhouette_pipeline: wgpu::RenderPipeline,
    ghost_pipeline: wgpu::RenderPipeline,
    /// Cast shadows on the ground, drawn after the sea and before sprites.
    shadow_pipeline: wgpu::RenderPipeline,
    sheets: Vec<Sheet>,
    sprite_buffer: wgpu::Buffer,
    decal_pipeline: wgpu::RenderPipeline,
    decal_buffer: wgpu::Buffer,
    post_layout: wgpu::BindGroupLayout,
    post_sampler: wgpu::Sampler,
    post_uniforms: [wgpu::Buffer; 3],
    ink_pipeline: wgpu::RenderPipeline,
    blur_pipeline: wgpu::RenderPipeline,
    final_pipeline: wgpu::RenderPipeline,
    targets: Targets,
    encode_srgb: bool,
    hud_pipeline: wgpu::RenderPipeline,
    hud_group: wgpu::BindGroup,
    hud_uniform: wgpu::Buffer,
    hud_buffer: wgpu::Buffer,
}

impl Renderer {
    pub fn new(gpu: &Gpu, assets: &Assets, sheets: &[Rgba], hud_atlas: &Rgba) -> Self {
        let device = &gpu.device;
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let cells = data_texture(device, wgpu::TextureFormat::Rgba8Unorm, "cells", 120, 80);
        let ground_index = data_texture(
            device,
            wgpu::TextureFormat::Rg8Unorm,
            "ground index and building plots",
            120,
            80,
        );
        let linear = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("globals"),
            entries: &[
                uniform_entry(0),
                texture_entry(
                    1,
                    wgpu::TextureViewDimension::D2,
                    wgpu::TextureSampleType::Float { filterable: true },
                ),
                sampler_entry(2, wgpu::SamplerBindingType::Filtering),
                texture_entry(
                    3,
                    wgpu::TextureViewDimension::D2,
                    wgpu::TextureSampleType::Float { filterable: false },
                ),
            ],
        });
        let heights = data_texture(device, wgpu::TextureFormat::R32Float, "heights", 1, 1);
        let globals_group =
            globals_bind_group(device, &globals_layout, &globals, &cells, &linear, &heights);

        // Ground: painted biome textures as one mipmapped array.
        let mut layers: Vec<Rgba> = terrain::GROUND_LAYERS
            .iter()
            .map(|name| {
                assets
                    .image(&format!("terrain/{name}.webp"))
                    .resized(GROUND_SIZE, GROUND_SIZE)
            })
            .collect();
        // Layer 10 is the existing painted cobblestone surface for claimed plots.
        layers.push(
            assets
                .image("terrain/cobblestone.png")
                .resized(GROUND_SIZE, GROUND_SIZE),
        );
        // Dedicated road materials follow the plot layer without changing biomes.
        for name in ["road_dirt", "road_stone"] {
            layers.push(
                assets
                    .image(&format!("terrain/{name}.png"))
                    .resized(GROUND_SIZE, GROUND_SIZE),
            );
        }
        let ground_layers = upload_texture(device, &gpu.queue, &layers, "ground layers");
        let repeat = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let terrain_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain"),
            entries: &[
                texture_entry(
                    0,
                    wgpu::TextureViewDimension::D2Array,
                    wgpu::TextureSampleType::Float { filterable: true },
                ),
                texture_entry(
                    1,
                    wgpu::TextureViewDimension::D2,
                    wgpu::TextureSampleType::Float { filterable: false },
                ),
                sampler_entry(2, wgpu::SamplerBindingType::Filtering),
            ],
        });
        let terrain_group = bind_group(
            device,
            "terrain",
            &terrain_layout,
            &[
                wgpu::BindingResource::TextureView(&ground_layers.create_view(
                    &wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::D2Array),
                        ..Default::default()
                    },
                )),
                wgpu::BindingResource::TextureView(&ground_index.create_view(&Default::default())),
                wgpu::BindingResource::Sampler(&repeat),
            ],
        );
        let terrain_module = shader(device, "terrain", include_str!("shaders/terrain.wgsl"));
        let ground_attributes =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32];
        let terrain_pipeline = pipeline(
            device,
            PipelineSpec {
                label: "terrain",
                module: &terrain_module,
                layouts: &[Some(&globals_layout), Some(&terrain_layout)],
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<GroundVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &ground_attributes,
                })],
                format: SCENE_FORMAT,
                blend: None,
                depth: Some((true, wgpu::CompareFunction::Less)),
                vs: "vs",
                fs: "fs",
            },
        );
        let ground_mesh = terrain::ground_mesh(&terrain::Heights::unknown());
        let ground = (
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ground"),
                contents: bytemuck::cast_slice(&ground_mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }),
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ground indices"),
                contents: bytemuck::cast_slice(&ground_mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            ground_mesh.indices.len() as u32,
        );

        let sea_module = shader(device, "sea", include_str!("shaders/sea.wgsl"));
        let sea_attributes = wgpu::vertex_attr_array![0 => Float32x3];
        let sea_pipeline = pipeline(
            device,
            PipelineSpec {
                label: "sea",
                module: &sea_module,
                layouts: &[Some(&globals_layout)],
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 12,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &sea_attributes,
                })],
                format: SCENE_FORMAT,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                depth: Some((true, wgpu::CompareFunction::Less)),
                vs: "vs",
                fs: "fs",
            },
        );
        let sea_mesh = terrain::sea_mesh();
        let sea = (
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("sea"),
                contents: bytemuck::cast_slice(&sea_mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("sea indices"),
                contents: bytemuck::cast_slice(&sea_mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            sea_mesh.indices.len() as u32,
        );

        let sky_module = shader(device, "sky", include_str!("shaders/sky.wgsl"));
        let sky_pipeline = pipeline(
            device,
            PipelineSpec {
                label: "sky",
                module: &sky_module,
                layouts: &[Some(&globals_layout)],
                buffers: &[],
                format: SCENE_FORMAT,
                blend: None,
                depth: Some((false, wgpu::CompareFunction::Always)),
                vs: "vs",
                fs: "fs",
            },
        );

        // Sprites: one bind group per generated sheet.
        let sheet_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sheet"),
            entries: &[
                texture_entry(
                    0,
                    wgpu::TextureViewDimension::D2,
                    wgpu::TextureSampleType::Float { filterable: true },
                ),
                sampler_entry(1, wgpu::SamplerBindingType::Filtering),
            ],
        });
        let sprite_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let sheets = sheets
            .iter()
            .map(|image| {
                let texture =
                    upload_texture(device, &gpu.queue, std::slice::from_ref(image), "sheet");
                Sheet {
                    bind_group: bind_group(
                        device,
                        "sheet",
                        &sheet_layout,
                        &[
                            wgpu::BindingResource::TextureView(
                                &texture.create_view(&Default::default()),
                            ),
                            wgpu::BindingResource::Sampler(&sprite_sampler),
                        ],
                    ),
                }
            })
            .collect();
        let sprite_module = shader(device, "billboard", include_str!("shaders/billboard.wgsl"));
        let sprite_attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x2, 3 => Float32x4, 4 => Float32, 5 => Float32x4, 6 => Float32x2, 7 => Float32x2];
        let make_sprite_pipeline = |ghost: bool| {
            pipeline(
                device,
                PipelineSpec {
                    label: if ghost { "building ghost" } else { "billboard" },
                    module: &sprite_module,
                    layouts: &[Some(&globals_layout), Some(&sheet_layout)],
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Sprite>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &sprite_attributes,
                    })],
                    format: SCENE_FORMAT,
                    blend: if ghost {
                        Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::OVER,
                            // Scene alpha is grading weight. Ghosts replace that
                            // weight proportionally without adding a matte.
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::Zero,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        })
                    } else {
                        None
                    },
                    depth: Some((
                        !ghost,
                        if ghost {
                            wgpu::CompareFunction::Always
                        } else {
                            wgpu::CompareFunction::Less
                        },
                    )),
                    vs: "vs",
                    fs: "fs",
                },
            )
        };
        let sprite_pipeline = make_sprite_pipeline(false);
        let ghost_pipeline = make_sprite_pipeline(true);
        let shadow_pipeline = pipeline(
            device,
            PipelineSpec {
                label: "shadow",
                module: &sprite_module,
                layouts: &[Some(&globals_layout), Some(&sheet_layout)],
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Sprite>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &sprite_attributes,
                })],
                format: SCENE_FORMAT,
                // Multiply the ground colour; scene alpha (grading weight) stays.
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Dst,
                        dst_factor: wgpu::BlendFactor::Zero,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                depth: Some((true, wgpu::CompareFunction::Less)),
                vs: "vs_shadow",
                fs: "fs_shadow",
            },
        );
        let silhouette_pipeline = pipeline(
            device,
            PipelineSpec {
                label: "silhouette",
                module: &sprite_module,
                layouts: &[Some(&globals_layout), Some(&sheet_layout)],
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Sprite>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &sprite_attributes,
                })],
                format: SCENE_FORMAT,
                // Tint the colour only; scene alpha (the grading weight) stays.
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::OVER,
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                depth: Some((false, wgpu::CompareFunction::Greater)),
                vs: "vs",
                fs: "fs_silhouette",
            },
        );
        let decal_module = shader(device, "decal", include_str!("shaders/decal.wgsl"));
        let decal_attributes =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32, 2 => Float32x4, 3 => Float32];
        let decal_pipeline = pipeline(
            device,
            PipelineSpec {
                label: "decal",
                module: &decal_module,
                layouts: &[Some(&globals_layout)],
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Decal>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &decal_attributes,
                })],
                format: SCENE_FORMAT,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                depth: Some((false, wgpu::CompareFunction::LessEqual)),
                vs: "vs",
                fs: "fs",
            },
        );
        let sprite_buffer = instance_buffer(device, 64 * 1024);
        let decal_buffer = instance_buffer(device, 16 * 1024);

        // Finish passes.
        let post_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post"),
            entries: &[
                uniform_entry(0),
                texture_entry(
                    1,
                    wgpu::TextureViewDimension::D2,
                    wgpu::TextureSampleType::Float { filterable: true },
                ),
                sampler_entry(2, wgpu::SamplerBindingType::Filtering),
                texture_entry(
                    3,
                    wgpu::TextureViewDimension::D2,
                    wgpu::TextureSampleType::Float { filterable: false },
                ),
            ],
        });
        let post_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("post"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/post.wgsl").into()),
        });
        let post_pipeline = |label, fs, format| {
            pipeline(
                device,
                PipelineSpec {
                    label,
                    module: &post_module,
                    layouts: &[Some(&post_layout)],
                    buffers: &[],
                    format,
                    blend: None,
                    depth: None,
                    vs: "vs",
                    fs,
                },
            )
        };
        let ink_pipeline = post_pipeline("ink", "ink", SCENE_FORMAT);
        let blur_pipeline = post_pipeline("blur", "blur", SCENE_FORMAT);
        let final_pipeline = post_pipeline("finish", "blur", gpu.config.format);
        let post_uniforms = [0, 1, 2].map(|_| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("post"),
                size: std::mem::size_of::<PostUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });
        let post_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let encode_srgb = !gpu.config.format.is_srgb();
        // Interface: one atlas, drawn last straight onto the surface.
        let hud_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("hud"),
            entries: &[
                uniform_entry(0),
                texture_entry(
                    1,
                    wgpu::TextureViewDimension::D2,
                    wgpu::TextureSampleType::Float { filterable: true },
                ),
                sampler_entry(2, wgpu::SamplerBindingType::Filtering),
                texture_entry(
                    3,
                    wgpu::TextureViewDimension::D2,
                    wgpu::TextureSampleType::Float { filterable: true },
                ),
            ],
        });
        let atlas = upload_texture(
            device,
            &gpu.queue,
            std::slice::from_ref(hud_atlas),
            "hud atlas",
        );
        let hud_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hud"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let hud_group = bind_group(
            device,
            "hud",
            &hud_layout,
            &[
                hud_uniform.as_entire_binding(),
                wgpu::BindingResource::TextureView(&atlas.create_view(&Default::default())),
                wgpu::BindingResource::Sampler(&sprite_sampler),
                wgpu::BindingResource::TextureView(&cells.create_view(&Default::default())),
            ],
        );
        let hud_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hud"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/hud.wgsl").into()),
        });
        let hud_attributes = wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4];
        let hud_pipeline = pipeline(
            device,
            PipelineSpec {
                label: "hud",
                module: &hud_module,
                layouts: &[Some(&hud_layout)],
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Quad>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &hud_attributes,
                })],
                format: gpu.config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                depth: None,
                vs: "vs",
                fs: "fs",
            },
        );
        let hud_buffer = instance_buffer(device, 64 * 1024);
        let targets = Self::make_targets(
            device,
            &post_layout,
            &post_sampler,
            &post_uniforms,
            gpu.config.width,
            gpu.config.height,
        );
        Self {
            globals_layout,
            terrain_layout,
            hud_layout,
            linear,
            repeat,
            sprite_sampler,
            ground_layers,
            atlas,
            ground_bounds: [0.0; 5],
            globals,
            globals_group,
            cells,
            heights,
            ground_index,
            terrain_pipeline,
            terrain_group,
            ground,
            sea_pipeline,
            sea,
            sky_pipeline,
            sprite_pipeline,
            silhouette_pipeline,
            ghost_pipeline,
            shadow_pipeline,
            sheets,
            sprite_buffer,
            decal_pipeline,
            decal_buffer,
            post_layout,
            post_sampler,
            post_uniforms,
            ink_pipeline,
            blur_pipeline,
            final_pipeline,
            targets,
            encode_srgb,
            hud_pipeline,
            hud_group,
            hud_uniform,
            hud_buffer,
        }
    }

    fn make_targets(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        uniforms: &[wgpu::Buffer; 3],
        width: u32,
        height: u32,
    ) -> Targets {
        let depth = render_target(device, DEPTH_FORMAT, width, height);
        let scene = render_target(device, SCENE_FORMAT, width, height);
        let inked = render_target(device, SCENE_FORMAT, width, height);
        let half = render_target(device, SCENE_FORMAT, width, height);
        let group = |uniform: &wgpu::Buffer, image: &wgpu::TextureView| {
            bind_group(
                device,
                "post",
                layout,
                &[
                    uniform.as_entire_binding(),
                    wgpu::BindingResource::TextureView(image),
                    wgpu::BindingResource::Sampler(sampler),
                    wgpu::BindingResource::TextureView(&depth),
                ],
            )
        };
        let ink_group = group(&uniforms[0], &scene);
        let blur_h_group = group(&uniforms[1], &inked);
        let blur_v_group = group(&uniforms[2], &half);
        Targets {
            depth,
            scene,
            inked,
            half,
            ink_group,
            blur_h_group,
            blur_v_group,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.targets = Self::make_targets(
            device,
            &self.post_layout,
            &self.post_sampler,
            &self.post_uniforms,
            width.max(1),
            height.max(1),
        );
    }

    /// Draws one frame. `sprites` holds (sheet index, sprite) pairs.
    pub fn render(
        &mut self,
        gpu: &Gpu,
        globals: &Globals,
        near_far: (f32, f32),
        sprites: &mut [(usize, Sprite)],
        decals: &[Decal],
        hud: &[Quad],
    ) {
        let frame = match gpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            _ => {
                gpu.surface.configure(&gpu.device, &gpu.config);
                return;
            }
        };
        let (width, height) = (gpu.config.width as f32, gpu.config.height as f32);
        gpu.queue.write_buffer(
            &self.globals,
            0,
            bytemuck::bytes_of(&Globals {
                ground: [
                    self.ground_bounds[0],
                    self.ground_bounds[1],
                    self.ground_bounds[2],
                    self.ground_bounds[3],
                ],
                ..*globals
            }),
        );
        let post = |step: [f32; 2], final_pass: bool| PostUniform {
            texel: [1.0 / width, 1.0 / height],
            near: near_far.0,
            far: near_far.1,
            step,
            encode_srgb: if self.encode_srgb { 1.0 } else { 0.0 },
            final_pass: if final_pass { 1.0 } else { 0.0 },
        };
        gpu.queue.write_buffer(
            &self.post_uniforms[0],
            0,
            bytemuck::bytes_of(&post([0.0, 0.0], false)),
        );
        gpu.queue.write_buffer(
            &self.post_uniforms[1],
            0,
            bytemuck::bytes_of(&post([1.0 / width, 0.0], false)),
        );
        gpu.queue.write_buffer(
            &self.post_uniforms[2],
            0,
            bytemuck::bytes_of(&post([0.0, 1.0 / height], true)),
        );

        sprites.sort_by_key(|(sheet, sprite)| (sprite.tint[3] < 1.0, *sheet));
        let instances: Vec<Sprite> = sprites.iter().map(|(_, sprite)| *sprite).collect();
        ensure_capacity(
            &gpu.device,
            &mut self.sprite_buffer,
            std::mem::size_of_val(instances.as_slice()),
        );
        gpu.queue
            .write_buffer(&self.sprite_buffer, 0, bytemuck::cast_slice(&instances));
        ensure_capacity(
            &gpu.device,
            &mut self.decal_buffer,
            std::mem::size_of_val(decals),
        );
        gpu.queue
            .write_buffer(&self.decal_buffer, 0, bytemuck::cast_slice(decals));
        ensure_capacity(
            &gpu.device,
            &mut self.hud_buffer,
            std::mem::size_of_val(hud),
        );
        gpu.queue
            .write_buffer(&self.hud_buffer, 0, bytemuck::cast_slice(hud));
        let hud_uniform = [
            width,
            height,
            if self.encode_srgb { 1.0 } else { 0.0 },
            0.0f32,
        ];
        gpu.queue
            .write_buffer(&self.hud_uniform, 0, bytemuck::cast_slice(&hud_uniform));

        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("world"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.targets.scene,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.targets.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.globals_group, &[]);
            pass.set_pipeline(&self.sky_pipeline);
            pass.draw(0..3, 0..1);
            pass.set_pipeline(&self.terrain_pipeline);
            pass.set_bind_group(1, &self.terrain_group, &[]);
            pass.set_vertex_buffer(0, self.ground.0.slice(..));
            pass.set_index_buffer(self.ground.1.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..self.ground.2, 0, 0..1);
            if !decals.is_empty() {
                pass.set_pipeline(&self.decal_pipeline);
                pass.set_vertex_buffer(0, self.decal_buffer.slice(..));
                pass.draw(0..6, 0..decals.len() as u32);
            }
            pass.set_pipeline(&self.sea_pipeline);
            pass.set_vertex_buffer(0, self.sea.0.slice(..));
            pass.set_index_buffer(self.sea.1.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..self.sea.2, 0, 0..1);
            pass.set_vertex_buffer(0, self.sprite_buffer.slice(..));
            pass.set_pipeline(&self.shadow_pipeline);
            let mut start = 0;
            while start < sprites.len() {
                let sheet = sprites[start].0;
                let building = sprites[start].1.footprint[0] > 0.0;
                let end = start
                    + sprites[start..]
                        .iter()
                        .take_while(|(s, sprite)| {
                            *s == sheet && (sprite.footprint[0] > 0.0) == building
                        })
                        .count();
                pass.set_bind_group(1, &self.sheets[sheet].bind_group, &[]);
                // Buildings sweep three draped box quads; units project one card.
                let vertices = if building {
                    3 * SHADOW_GRID * SHADOW_GRID * 6
                } else {
                    6
                };
                pass.draw(0..vertices, start as u32..end as u32);
                start = end;
            }
            let mut start = 0;
            while start < sprites.len() {
                let sheet = sprites[start].0;
                let ghost = sprites[start].1.tint[3] < 1.0;
                pass.set_pipeline(if ghost {
                    &self.ghost_pipeline
                } else {
                    &self.sprite_pipeline
                });
                let end = start
                    + sprites[start..]
                        .iter()
                        .take_while(|(s, sprite)| *s == sheet && (sprite.tint[3] < 1.0) == ghost)
                        .count();
                pass.set_bind_group(1, &self.sheets[sheet].bind_group, &[]);
                pass.draw(0..6, start as u32..end as u32);
                start = end;
            }
            // Then the parts of villagers that something nearer covers.
            pass.set_pipeline(&self.silhouette_pipeline);
            let mut start = 0;
            while start < sprites.len() {
                let sheet = sprites[start].0;
                let end = start
                    + sprites[start..]
                        .iter()
                        .take_while(|(s, _)| *s == sheet)
                        .count();
                if crate::view::VILLAGER_SHEETS.contains(&sheet) {
                    pass.set_bind_group(1, &self.sheets[sheet].bind_group, &[]);
                    pass.draw(0..6, start as u32..end as u32);
                }
                start = end;
            }
        }
        let surface_view = frame.texture.create_view(&Default::default());
        let passes: [(&wgpu::RenderPipeline, &wgpu::BindGroup, &wgpu::TextureView); 3] = [
            (
                &self.ink_pipeline,
                &self.targets.ink_group,
                &self.targets.inked,
            ),
            (
                &self.blur_pipeline,
                &self.targets.blur_h_group,
                &self.targets.half,
            ),
            (
                &self.final_pipeline,
                &self.targets.blur_v_group,
                &surface_view,
            ),
        ];
        for (pipeline, group, target) in passes {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("finish"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.draw(0..3, 0..1);
        }
        if !hud.is_empty() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hud"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.hud_pipeline);
            pass.set_bind_group(0, &self.hud_group, &[]);
            pass.set_vertex_buffer(0, self.hud_buffer.slice(..));
            pass.draw(0..6, 0..hud.len() as u32);
        }
        gpu.queue.submit([encoder.finish()]);
        gpu.queue.present(frame);
    }
}
