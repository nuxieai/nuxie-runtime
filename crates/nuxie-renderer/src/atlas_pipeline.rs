//! Offscreen feather-mask rendering translated from Rive's atlas pipeline.

use crate::gpu::{
    ContourData, FlushUniforms, PaintAuxData, PaintData, PatchVertex, PathData,
    MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT, MIDPOINT_FAN_PATCH_INDEX_COUNT,
};
use wgpu::util::DeviceExt;

pub(crate) struct AtlasPipeline {
    fill: wgpu::RenderPipeline,
    stroke: wgpu::RenderPipeline,
    flush_layout: wgpu::BindGroupLayout,
    image_layout: wgpu::BindGroupLayout,
    sampler_layout: wgpu::BindGroupLayout,
}

impl AtlasPipeline {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let vertex = shader(
            device,
            "nuxie-atlas-vertex",
            include_str!("generated/render_atlas.webgpu_vert.wgsl"),
        );
        let fill_fragment = shader(
            device,
            "nuxie-atlas-fill-fragment",
            include_str!("generated/render_atlas_fill.webgpu_frag.wgsl"),
        );
        let stroke_fragment = shader(
            device,
            "nuxie-atlas-stroke-fragment",
            include_str!("generated/render_atlas_stroke.webgpu_frag.wgsl"),
        );
        let flush_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-atlas-flush-layout"),
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
            label: Some("nuxie-atlas-image-layout"),
            entries: &[
                texture_entry(12, wgpu::TextureSampleType::Float { filterable: true }),
                sampler_entry(14),
            ],
        });
        let empty_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-atlas-empty-layout"),
            entries: &[],
        });
        let sampler_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-atlas-sampler-layout"),
            entries: &[sampler_entry(9), sampler_entry(10)],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nuxie-atlas-pipeline-layout"),
            bind_group_layouts: &[
                Some(&flush_layout),
                Some(&image_layout),
                Some(&empty_layout),
                Some(&sampler_layout),
            ],
            immediate_size: 0,
        });
        let make_pipeline = |label, fragment: &wgpu::ShaderModule, operation| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &vertex,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(PatchVertex::layout())],
                },
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Cw,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: fragment,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::R16Float,
                        blend: Some(wgpu::BlendState {
                            color: blend_component(operation),
                            alpha: blend_component(operation),
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let fill = make_pipeline(
            "nuxie-atlas-fill-pipeline",
            &fill_fragment,
            wgpu::BlendOperation::Add,
        );
        let stroke = make_pipeline(
            "nuxie-atlas-stroke-pipeline",
            &stroke_fragment,
            wgpu::BlendOperation::Max,
        );
        Self {
            fill,
            stroke,
            flush_layout,
            image_layout,
            sampler_layout,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_mask(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        patch_vertices: &wgpu::Buffer,
        patch_indices: &wgpu::Buffer,
        tessellation: &wgpu::TextureView,
        feather_lut: &wgpu::TextureView,
        uniforms: &FlushUniforms,
        paths: &[PathData],
        paints: &[PaintData],
        paint_aux: &[PaintAuxData],
        contours: &[ContourData],
        base_instance: u32,
        instance_count: u32,
        is_stroke: bool,
        clear: bool,
        scissor: [u32; 4],
    ) {
        let uniform = upload(
            device,
            "nuxie-atlas-uniforms",
            std::slice::from_ref(uniforms),
        );
        let path = upload(device, "nuxie-atlas-path", paths);
        let paint = upload(device, "nuxie-atlas-paint", paints);
        let paint_aux = upload(device, "nuxie-atlas-paint-aux", paint_aux);
        let contours = upload(device, "nuxie-atlas-contours", contours);
        let dummy = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-atlas-dummy-texture"),
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
            label: Some("nuxie-atlas-linear-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let flush = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-atlas-flush-group"),
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
            ],
        });
        let image = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-atlas-image-group"),
            layout: &self.image_layout,
            entries: &[
                binding(12, wgpu::BindingResource::TextureView(&dummy_view)),
                binding(14, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        let samplers = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-atlas-sampler-group"),
            layout: &self.sampler_layout,
            entries: &[
                binding(9, wgpu::BindingResource::Sampler(&sampler)),
                binding(10, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        let attachments = [Some(wgpu::RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: if clear {
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                } else {
                    wgpu::LoadOp::Load
                },
                store: wgpu::StoreOp::Store,
            },
        })];
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("nuxie-atlas-mask-pass"),
            color_attachments: &attachments,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(if is_stroke { &self.stroke } else { &self.fill });
        pass.set_scissor_rect(scissor[0], scissor[1], scissor[2], scissor[3]);
        pass.set_bind_group(0, &flush, &[]);
        pass.set_bind_group(1, &image, &[]);
        pass.set_bind_group(3, &samplers, &[]);
        pass.set_vertex_buffer(0, patch_vertices.slice(..));
        pass.set_index_buffer(patch_indices.slice(..), wgpu::IndexFormat::Uint16);
        let index_range = if is_stroke {
            0..48
        } else {
            MIDPOINT_FAN_PATCH_INDEX_COUNT as u32
                ..(MIDPOINT_FAN_PATCH_INDEX_COUNT + MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT) as u32
        };
        pass.draw_indexed(
            index_range,
            0,
            base_instance..base_instance + instance_count,
        );
    }
}

fn blend_component(operation: wgpu::BlendOperation) -> wgpu::BlendComponent {
    wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation,
    }
}

fn shader(device: &wgpu::Device, label: &'static str, source: &'static str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

fn upload<T: bytemuck::Pod>(
    device: &wgpu::Device,
    label: &'static str,
    values: &[T],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(values),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE,
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
