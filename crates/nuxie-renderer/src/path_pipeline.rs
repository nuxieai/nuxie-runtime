//! MSAA analytic path draw translated from `draw_path.vert` and `draw_msaa_object.frag`.

use crate::gpu::{ContourData, FlushUniforms, PaintAuxData, PaintData, PatchVertex, PathData};
use wgpu::util::DeviceExt;

pub(crate) struct PathPipeline {
    pub pipeline: wgpu::RenderPipeline,
    flush_layout: wgpu::BindGroupLayout,
    image_layout: wgpu::BindGroupLayout,
    sampler_layout: wgpu::BindGroupLayout,
}

pub(crate) struct PreparedPathDraw {
    pub flush_group: wgpu::BindGroup,
    pub image_group: wgpu::BindGroup,
    pub sampler_group: wgpu::BindGroup,
    pub base_instance: u32,
    pub instance_count: u32,
}

impl PathPipeline {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let vertex = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nuxie-msaa-path-vertex"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("generated/draw_msaa_path.webgpu_noclipdistance_vert.wgsl").into(),
            ),
        });
        let fragment = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nuxie-msaa-path-fragment"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("generated/draw_msaa_path.webgpu_fixedcolor_frag.wgsl").into(),
            ),
        });
        let flush_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-msaa-path-flush-layout"),
            entries: &[
                uniform_entry(0),
                storage_entry(3),
                storage_entry(4),
                storage_entry(5),
                storage_entry(6),
                texture_entry(8, wgpu::TextureSampleType::Uint),
                texture_entry(9, wgpu::TextureSampleType::Float { filterable: true }),
                texture_entry(10, wgpu::TextureSampleType::Float { filterable: true }),
            ],
        });
        let image_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-msaa-path-image-layout"),
            entries: &[
                texture_entry(12, wgpu::TextureSampleType::Float { filterable: true }),
                sampler_entry(14),
            ],
        });
        let empty_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-msaa-path-empty-layout"),
            entries: &[],
        });
        let sampler_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-msaa-path-sampler-layout"),
            entries: &[sampler_entry(9), sampler_entry(10)],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nuxie-msaa-path-pipeline-layout"),
            bind_group_layouts: &[
                Some(&flush_layout),
                Some(&image_layout),
                Some(&empty_layout),
                Some(&sampler_layout),
            ],
            immediate_size: 0,
        });
        let vertex_options = wgpu::PipelineCompilationOptions {
            constants: &[("0", 0.0), ("2", 0.0)],
            ..Default::default()
        };
        let fragment_options = wgpu::PipelineCompilationOptions {
            constants: &[("2", 0.0), ("7", 0.0)],
            ..Default::default()
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-msaa-path-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vertex,
                entry_point: Some("main"),
                compilation_options: vertex_options,
                buffers: &[Some(PatchVertex::layout())],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Stencil8,
                depth_write_enabled: None,
                depth_compare: None,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment,
                entry_point: Some("main"),
                compilation_options: fragment_options,
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            flush_layout,
            image_layout,
            sampler_layout,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &self,
        device: &wgpu::Device,
        tessellation_view: &wgpu::TextureView,
        uniforms: &FlushUniforms,
        path: &PathData,
        paint: &PaintData,
        paint_aux: &PaintAuxData,
        contours: &[ContourData],
        base_instance: u32,
        instance_count: u32,
    ) -> PreparedPathDraw {
        let uniform_buffer = upload(
            device,
            "nuxie-path-uniforms",
            std::slice::from_ref(uniforms),
            wgpu::BufferUsages::UNIFORM,
        );
        let path_buffer = upload(
            device,
            "nuxie-path-data",
            std::slice::from_ref(path),
            wgpu::BufferUsages::STORAGE,
        );
        let paint_buffer = upload(
            device,
            "nuxie-paint-data",
            std::slice::from_ref(paint),
            wgpu::BufferUsages::STORAGE,
        );
        let paint_aux_buffer = upload(
            device,
            "nuxie-paint-aux-data",
            std::slice::from_ref(paint_aux),
            wgpu::BufferUsages::STORAGE,
        );
        let contour_buffer = upload(
            device,
            "nuxie-contour-data",
            contours,
            wgpu::BufferUsages::STORAGE,
        );
        let dummy = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-path-dummy-texture"),
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
        let dummy_view = dummy.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let flush_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-msaa-path-flush-group"),
            layout: &self.flush_layout,
            entries: &[
                binding(0, uniform_buffer.as_entire_binding()),
                binding(3, path_buffer.as_entire_binding()),
                binding(4, paint_buffer.as_entire_binding()),
                binding(5, paint_aux_buffer.as_entire_binding()),
                binding(6, contour_buffer.as_entire_binding()),
                binding(8, wgpu::BindingResource::TextureView(tessellation_view)),
                binding(9, wgpu::BindingResource::TextureView(&dummy_view)),
                binding(10, wgpu::BindingResource::TextureView(&dummy_view)),
            ],
        });
        let image_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-msaa-path-image-group"),
            layout: &self.image_layout,
            entries: &[
                binding(12, wgpu::BindingResource::TextureView(&dummy_view)),
                binding(14, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        let sampler_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-msaa-path-sampler-group"),
            layout: &self.sampler_layout,
            entries: &[
                binding(9, wgpu::BindingResource::Sampler(&sampler)),
                binding(10, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        PreparedPathDraw {
            flush_group,
            image_group,
            sampler_group,
            base_instance,
            instance_count,
        }
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

fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
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

fn texture_entry(binding: u32, sample_type: wgpu::TextureSampleType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}
