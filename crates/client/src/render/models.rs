//! Generated 3D building models (assets/models): a compact vertex/index blob
//! plus an albedo texture per model, drawn on the plot in place of the sprite.
use super::*;

/// Models the client loads, in the order `ModelInstance::model` indexes.
pub const MODEL_NAMES: [&str; 4] = ["towncenter", "house", "watchtower", "monument"];

/// One placed model: the plot's centre on the ground and the plot size the
/// model's footprint should fill.
#[derive(Clone, Copy, Debug)]
pub struct ModelInstance {
    pub model: usize,
    pub translation: [f32; 3],
    pub footprint: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ModelGpuInstance {
    pub translation: [f32; 3],
    pub scale: f32,
}

pub struct Model {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
    /// World extent of the unit-scaled mesh (x, y, z).
    extent: [f32; 3],
    bind_group: wgpu::BindGroup,
}

pub struct Models {
    models: Vec<Model>,
    pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    instances: wgpu::Buffer,
}

impl Models {
    pub fn new(
        gpu: &Gpu,
        assets: &Assets,
        globals_layout: &wgpu::BindGroupLayout,
        sheet_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
    ) -> Self {
        let device = &gpu.device;
        let models = MODEL_NAMES
            .iter()
            .map(|name| {
                let bytes = assets.bytes(&format!("models/{name}.bin"));
                let count =
                    |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
                let (vertex_count, index_count) = (count(0), count(4));
                let vertex_bytes = vertex_count * 8 * 4;
                let vertex_data = &bytes[8..8 + vertex_bytes];
                let index_data = &bytes[8 + vertex_bytes..8 + vertex_bytes + index_count * 4];
                let positions: Vec<f32> = vertex_data
                    .chunks_exact(4)
                    .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                    .collect();
                let mut low = [f32::MAX; 3];
                let mut high = [f32::MIN; 3];
                for vertex in positions.chunks_exact(8) {
                    for axis in 0..3 {
                        low[axis] = low[axis].min(vertex[axis]);
                        high[axis] = high[axis].max(vertex[axis]);
                    }
                }
                let texture = upload_texture(
                    device,
                    &gpu.queue,
                    &[assets.image(&format!("models/{name}.png"))],
                    name,
                );
                Model {
                    vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(name),
                        contents: vertex_data,
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                    indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(name),
                        contents: index_data,
                        usage: wgpu::BufferUsages::INDEX,
                    }),
                    index_count: index_count as u32,
                    extent: [high[0] - low[0], high[1] - low[1], high[2] - low[2]],
                    bind_group: bind_group(
                        device,
                        name,
                        sheet_layout,
                        &[
                            wgpu::BindingResource::TextureView(
                                &texture.create_view(&Default::default()),
                            ),
                            wgpu::BindingResource::Sampler(sampler),
                        ],
                    ),
                }
            })
            .collect();
        let module = shader(device, "model", include_str!("../shaders/model.wgsl"));
        let vertex_attributes =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];
        let instance_attributes = wgpu::vertex_attr_array![3 => Float32x3, 4 => Float32];
        let buffers = [
            Some(wgpu::VertexBufferLayout {
                array_stride: 8 * 4,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &vertex_attributes,
            }),
            Some(wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<ModelGpuInstance>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &instance_attributes,
            }),
        ];
        let make = |label, blend, vs, fs| {
            pipeline(
                device,
                PipelineSpec {
                    label,
                    module: &module,
                    layouts: &[Some(globals_layout), Some(sheet_layout)],
                    buffers: &buffers,
                    format: SCENE_FORMAT,
                    blend,
                    depth: Some((true, wgpu::CompareFunction::Less)),
                    vs,
                    fs,
                },
            )
        };
        Self {
            models,
            pipeline: make("model", None, "vs", "fs"),
            shadow_pipeline: make(
                "model shadow",
                Some(wgpu::BlendState {
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
                "vs_shadow",
                "fs_shadow",
            ),
            instances: instance_buffer(device, 4 * 1024),
        }
    }

    /// Uploads the frame's instances sorted by model; returns per-model ranges.
    pub fn upload(&mut self, gpu: &Gpu, instances: &mut [ModelInstance]) -> Vec<(usize, u32, u32)> {
        instances.sort_by_key(|instance| instance.model);
        let gpu_instances: Vec<ModelGpuInstance> = instances
            .iter()
            .map(|instance| {
                let extent = self.models[instance.model].extent;
                ModelGpuInstance {
                    translation: instance.translation,
                    scale: (instance.footprint[0] / extent[0])
                        .min(instance.footprint[1] / extent[2]),
                }
            })
            .collect();
        ensure_capacity(
            &gpu.device,
            &mut self.instances,
            std::mem::size_of_val(gpu_instances.as_slice()),
        );
        gpu.queue
            .write_buffer(&self.instances, 0, bytemuck::cast_slice(&gpu_instances));
        let mut ranges = Vec::new();
        let mut start = 0;
        while start < instances.len() {
            let model = instances[start].model;
            let end = start
                + instances[start..]
                    .iter()
                    .take_while(|instance| instance.model == model)
                    .count();
            ranges.push((model, start as u32, end as u32));
            start = end;
        }
        ranges
    }

    pub fn draw(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        ranges: &[(usize, u32, u32)],
        shadow: bool,
    ) {
        pass.set_pipeline(if shadow {
            &self.shadow_pipeline
        } else {
            &self.pipeline
        });
        pass.set_vertex_buffer(1, self.instances.slice(..));
        for &(model, start, end) in ranges {
            let model = &self.models[model];
            pass.set_bind_group(1, &model.bind_group, &[]);
            pass.set_vertex_buffer(0, model.vertices.slice(..));
            pass.set_index_buffer(model.indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..model.index_count, 0, start..end);
        }
    }
}
