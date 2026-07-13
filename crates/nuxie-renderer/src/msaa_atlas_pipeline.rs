//! Fixed-function MSAA atlas blit translated from Rive's WebGPU renderer.

use crate::gpu::{ContourData, FlushUniforms, PaintAuxData, PaintData, PathData, TriangleVertex};
use wgpu::util::DeviceExt;

pub(crate) struct MsaaAtlasPipeline {
    no_clip_pipeline: wgpu::RenderPipeline,
    clip_rect_pipeline: Option<wgpu::RenderPipeline>,
    flush_layout: wgpu::BindGroupLayout,
    image_layout: wgpu::BindGroupLayout,
    sampler_layout: wgpu::BindGroupLayout,
}

pub(crate) struct PreparedAtlasBlit {
    pub flush_group: wgpu::BindGroup,
    pub image_group: wgpu::BindGroup,
    pub sampler_group: wgpu::BindGroup,
    pub vertices: wgpu::Buffer,
    pub vertex_count: u32,
    clipped: bool,
}

impl MsaaAtlasPipeline {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let no_clip_vertex = shader(
            device,
            "nuxie-msaa-atlas-blit-vertex",
            include_str!("generated/draw_msaa_atlas_blit.webgpu_noclipdistance_vert.wgsl"),
        );
        let fragment = shader(
            device,
            "nuxie-msaa-atlas-blit-fragment",
            include_str!("generated/draw_msaa_atlas_blit.webgpu_fixedcolor_frag.wgsl"),
        );
        let flush_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-msaa-atlas-flush-layout"),
            entries: &[
                uniform_entry(0),
                storage_entry(3),
                storage_entry(4),
                storage_entry(5),
                storage_entry(6),
                texture_entry(8, wgpu::TextureSampleType::Uint),
                texture_entry(9, wgpu::TextureSampleType::Float { filterable: true }),
                texture_entry(10, wgpu::TextureSampleType::Float { filterable: true }),
                texture_entry(11, wgpu::TextureSampleType::Float { filterable: true }),
            ],
        });
        let image_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-msaa-atlas-image-layout"),
            entries: &[
                texture_entry(12, wgpu::TextureSampleType::Float { filterable: true }),
                sampler_entry(14),
            ],
        });
        let empty_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-msaa-atlas-empty-layout"),
            entries: &[],
        });
        let sampler_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-msaa-atlas-sampler-layout"),
            entries: &[sampler_entry(9), sampler_entry(10), sampler_entry(11)],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nuxie-msaa-atlas-pipeline-layout"),
            bind_group_layouts: &[
                Some(&flush_layout),
                Some(&image_layout),
                Some(&empty_layout),
                Some(&sampler_layout),
            ],
            immediate_size: 0,
        });
        let create_pipeline = |label, vertex: &wgpu::ShaderModule, clipped| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: vertex,
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: if clipped {
                            &[("0", 0.0), ("1", 1.0), ("2", 0.0)]
                        } else {
                            &[("0", 0.0), ("2", 0.0)]
                        },
                        ..Default::default()
                    },
                    buffers: &[Some(TriangleVertex::layout())],
                },
                primitive: wgpu::PrimitiveState {
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
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[("2", 0.0), ("7", 1.0)],
                        ..Default::default()
                    },
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let no_clip_pipeline =
            create_pipeline("nuxie-msaa-atlas-blit-pipeline", &no_clip_vertex, false);
        let clip_rect_pipeline = device
            .features()
            .contains(wgpu::Features::CLIP_DISTANCES)
            .then(|| {
                let vertex = shader(
                    device,
                    "nuxie-msaa-atlas-blit-clip-rect-vertex",
                    include_str!("generated/draw_msaa_atlas_blit.webgpu_vert.wgsl"),
                );
                create_pipeline("nuxie-msaa-atlas-blit-clip-rect-pipeline", &vertex, true)
            });
        Self {
            no_clip_pipeline,
            clip_rect_pipeline,
            flush_layout,
            image_layout,
            sampler_layout,
        }
    }

    pub(crate) fn supports_clip_rect(&self) -> bool {
        self.clip_rect_pipeline.is_some()
    }

    pub(crate) fn pipeline<'a>(&'a self, draw: &PreparedAtlasBlit) -> &'a wgpu::RenderPipeline {
        if draw.clipped {
            self.clip_rect_pipeline
                .as_ref()
                .expect("clipped atlas blit prepared without clip-distance pipeline")
        } else {
            &self.no_clip_pipeline
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &self,
        device: &wgpu::Device,
        tessellation: &wgpu::TextureView,
        feather_lut: &wgpu::TextureView,
        atlas: &wgpu::TextureView,
        uniforms: &FlushUniforms,
        paths: &[PathData],
        paints: &[PaintData],
        paint_aux: &[PaintAuxData],
        contours: &[ContourData],
        vertices: &[TriangleVertex],
        clipped: bool,
    ) -> PreparedAtlasBlit {
        let uniform = upload(
            device,
            "nuxie-msaa-atlas-uniforms",
            std::slice::from_ref(uniforms),
            wgpu::BufferUsages::UNIFORM,
        );
        let path = upload(
            device,
            "nuxie-msaa-atlas-paths",
            paths,
            wgpu::BufferUsages::STORAGE,
        );
        let paint = upload(
            device,
            "nuxie-msaa-atlas-paints",
            paints,
            wgpu::BufferUsages::STORAGE,
        );
        let paint_aux = upload(
            device,
            "nuxie-msaa-atlas-paint-aux",
            paint_aux,
            wgpu::BufferUsages::STORAGE,
        );
        let contours = upload(
            device,
            "nuxie-msaa-atlas-contours",
            contours,
            wgpu::BufferUsages::STORAGE,
        );
        let vertex_buffer = upload(
            device,
            "nuxie-msaa-atlas-vertices",
            vertices,
            wgpu::BufferUsages::VERTEX,
        );
        let dummy = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-msaa-atlas-dummy-texture"),
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
        let dummy_view = dummy.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nuxie-msaa-atlas-linear-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let flush_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-msaa-atlas-flush-group"),
            layout: &self.flush_layout,
            entries: &[
                binding(0, uniform.as_entire_binding()),
                binding(3, path.as_entire_binding()),
                binding(4, paint.as_entire_binding()),
                binding(5, paint_aux.as_entire_binding()),
                binding(6, contours.as_entire_binding()),
                binding(8, wgpu::BindingResource::TextureView(tessellation)),
                binding(9, wgpu::BindingResource::TextureView(&dummy_view)),
                binding(10, wgpu::BindingResource::TextureView(feather_lut)),
                binding(11, wgpu::BindingResource::TextureView(atlas)),
            ],
        });
        let image_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-msaa-atlas-image-group"),
            layout: &self.image_layout,
            entries: &[
                binding(12, wgpu::BindingResource::TextureView(&dummy_view)),
                binding(14, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        let sampler_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-msaa-atlas-sampler-group"),
            layout: &self.sampler_layout,
            entries: &[
                binding(9, wgpu::BindingResource::Sampler(&sampler)),
                binding(10, wgpu::BindingResource::Sampler(&sampler)),
                binding(11, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        PreparedAtlasBlit {
            flush_group,
            image_group,
            sampler_group,
            vertices: vertex_buffer,
            vertex_count: vertices.len() as u32,
            clipped,
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

fn shader(device: &wgpu::Device, label: &'static str, source: &'static str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

fn binding(binding: u32, resource: wgpu::BindingResource<'_>) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry { binding, resource }
}

fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    buffer_entry(binding, wgpu::BufferBindingType::Uniform)
}

fn storage_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    buffer_entry(
        binding,
        wgpu::BufferBindingType::Storage { read_only: true },
    )
}

fn buffer_entry(binding: u32, ty: wgpu::BufferBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty,
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
