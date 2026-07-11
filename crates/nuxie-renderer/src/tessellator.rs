//! GPU tessellation pass translated from `renderer/src/shaders/tessellate.glsl`.

use crate::gpu::{ContourData, FlushUniforms, PathData, TessVertexSpan};

pub(crate) struct Tessellator {
    pub pipeline: wgpu::RenderPipeline,
    pub flush_layout: wgpu::BindGroupLayout,
    pub sampler_layout: wgpu::BindGroupLayout,
    pub span_indices: wgpu::Buffer,
}

impl Tessellator {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let vertex = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nuxie-tessellate-vertex"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("generated/tessellate.webgpu_vert.wgsl").into(),
            ),
        });
        let fragment = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nuxie-tessellate-fragment"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("generated/tessellate.webgpu_frag.wgsl").into(),
            ),
        });
        let flush_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-tessellate-flush-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                storage_entry(3),
                storage_entry(6),
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let empty_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-tessellate-empty-layout"),
            entries: &[],
        });
        let sampler_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-tessellate-sampler-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 10,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nuxie-tessellate-pipeline-layout"),
            bind_group_layouts: &[
                Some(&flush_layout),
                Some(&empty_layout),
                Some(&empty_layout),
                Some(&sampler_layout),
            ],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-tessellate-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(TessVertexSpan::layout())],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &fragment,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba32Uint,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let span_indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("nuxie-tessellation-span-indices"),
            contents: bytemuck::cast_slice(&[0u16, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7]),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            pipeline,
            flush_layout,
            sampler_layout,
            span_indices,
        }
    }

    pub(crate) fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        spans: &[TessVertexSpan],
        uniforms: &FlushUniforms,
        paths: &[PathData],
        contours: &[ContourData],
        height: u32,
    ) -> wgpu::Texture {
        assert!(!spans.is_empty() && !paths.is_empty() && !contours.is_empty());
        let span_buffer = upload(
            device,
            "nuxie-tessellation-spans",
            spans,
            wgpu::BufferUsages::VERTEX,
        );
        let uniform_buffer = upload(
            device,
            "nuxie-tessellation-uniforms",
            std::slice::from_ref(uniforms),
            wgpu::BufferUsages::UNIFORM,
        );
        let path_buffer = upload(
            device,
            "nuxie-tessellation-paths",
            paths,
            wgpu::BufferUsages::STORAGE,
        );
        let contour_buffer = upload(
            device,
            "nuxie-tessellation-contours",
            contours,
            wgpu::BufferUsages::STORAGE,
        );
        let atlas = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-tessellation-dummy-atlas"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let atlas_view = atlas.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let flush_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-tessellation-flush-group"),
            layout: &self.flush_layout,
            entries: &[
                binding(0, uniform_buffer.as_entire_binding()),
                binding(3, path_buffer.as_entire_binding()),
                binding(6, contour_buffer.as_entire_binding()),
                binding(10, wgpu::BindingResource::TextureView(&atlas_view)),
            ],
        });
        let sampler_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-tessellation-sampler-group"),
            layout: &self.sampler_layout,
            entries: &[binding(10, wgpu::BindingResource::Sampler(&sampler))],
        });
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-tessellation-data"),
            size: wgpu::Extent3d {
                width: 2048,
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Uint,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("nuxie-tessellation-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &flush_group, &[]);
        pass.set_bind_group(3, &sampler_group, &[]);
        pass.set_vertex_buffer(0, span_buffer.slice(..));
        pass.set_index_buffer(self.span_indices.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..12, 0, 0..spans.len() as u32);
        drop(pass);
        texture
    }
}

fn upload<T: bytemuck::Pod>(
    device: &wgpu::Device,
    label: &'static str,
    values: &[T],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(values),
        usage,
    })
}

fn binding(binding: u32, resource: wgpu::BindingResource<'_>) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry { binding, resource }
}

fn storage_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

use wgpu::util::DeviceExt;
