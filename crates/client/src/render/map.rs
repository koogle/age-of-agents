//! Growing map textures and camera-bounded terrain geometry.
use super::*;

impl Renderer {
    pub fn ground_region(&self, rig: &crate::camera::Rig) -> [f32; 5] {
        let radius = (rig.distance * (rig.width / rig.height.max(1.0) + 1.0) * 0.4 + 16.0).ceil();
        let bounds = |center: f32, size: f32| {
            (((center - radius) / 16.0).floor() * 16.0 - 4.0).max(-4.0)
                ..(((center + radius) / 16.0).ceil() * 16.0 + 4.0).min(size + 4.0)
        };
        let x = bounds(rig.target.x, rig.map_size.x);
        let z = bounds(rig.target.z, rig.map_size.y);
        [
            x.start,
            z.start,
            x.end,
            z.end,
            if rig.distance < 45.0 { 4.0 } else { 2.0 },
        ]
    }

    pub fn update_ground(
        &mut self,
        device: &wgpu::Device,
        heights: &terrain::Heights,
        region: [f32; 5],
    ) {
        let mesh = terrain::ground_mesh_region(
            heights,
            [region[0], region[1], region[2], region[3]],
            region[4] as u32,
        );
        self.ground = (
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("visible ground"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("visible ground indices"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            mesh.indices.len() as u32,
        );
        self.ground_bounds = region;
    }

    fn resize_map(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.cells.width() == width && self.cells.height() == height {
            return;
        }
        self.cells = data_texture(
            device,
            wgpu::TextureFormat::Rgba8Unorm,
            "cells",
            width,
            height,
        );
        self.ground_index = data_texture(
            device,
            wgpu::TextureFormat::Rg8Unorm,
            "ground index",
            width,
            height,
        );
        self.globals_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("map globals"),
            layout: &self.globals_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.globals.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &self.cells.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.linear),
                },
            ],
        });
        self.terrain_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("map terrain"),
            layout: &self.terrain_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.ground_layers.create_view(
                        &wgpu::TextureViewDescriptor {
                            dimension: Some(wgpu::TextureViewDimension::D2Array),
                            ..Default::default()
                        },
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &self.ground_index.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.repeat),
                },
            ],
        });
        self.hud_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("map hud"),
            layout: &self.hud_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.hud_uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &self.atlas.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sprite_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        &self.cells.create_view(&Default::default()),
                    ),
                },
            ],
        });
    }

    pub fn update_cells(&mut self, gpu: &Gpu, rgba: &[u8], layers: &[u8], columns: u16, rows: u16) {
        let (width, height) = (u32::from(columns), u32::from(rows));
        self.resize_map(&gpu.device, width, height);
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        for (texture, data, bytes) in [(&self.cells, rgba, 4), (&self.ground_index, layers, 2)] {
            gpu.queue.write_texture(
                texture.as_image_copy(),
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * bytes),
                    rows_per_image: Some(height),
                },
                size,
            );
        }
    }
}
