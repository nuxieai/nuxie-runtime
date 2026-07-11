//! Clockwise-atomic draw and resolve translated from Rive's WebGPU shaders.

use crate::gpu::{ContourData, FlushUniforms, PaintAuxData, PaintData, PatchVertex, PathData};
use wgpu::util::DeviceExt;

pub(crate) struct AtomicPipeline {
    path: wgpu::RenderPipeline,
    resolve: wgpu::RenderPipeline,
    flush_layout: wgpu::BindGroupLayout,
    image_layout: wgpu::BindGroupLayout,
    atomic_layout: wgpu::BindGroupLayout,
    sampler_layout: wgpu::BindGroupLayout,
}

impl AtomicPipeline {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let path_vertex = shader(
            device,
            "nuxie-atomic-path-vertex",
            include_str!("generated/atomic_draw_path.webgpu_vert.wgsl"),
        );
        let path_fragment = shader(
            device,
            "nuxie-atomic-path-fragment",
            include_str!("generated/atomic_draw_path.webgpu_fixedcolor_frag.wgsl"),
        );
        let resolve_vertex = shader(
            device,
            "nuxie-atomic-resolve-vertex",
            include_str!("generated/atomic_resolve.webgpu_vert.wgsl"),
        );
        let resolve_fragment = shader(
            device,
            "nuxie-atomic-resolve-fragment",
            include_str!("generated/atomic_resolve.webgpu_fixedcolor_frag.wgsl"),
        );
        let flush_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-atomic-flush-layout"),
            entries: &[
                uniform_entry(0),
                storage_entry(3, true),
                storage_entry(4, true),
                storage_entry(5, true),
                storage_entry(6, true),
                texture_entry(8, wgpu::TextureSampleType::Uint),
                texture_entry(9, wgpu::TextureSampleType::Float { filterable: true }),
                texture_entry(10, wgpu::TextureSampleType::Float { filterable: true }),
            ],
        });
        let image_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-atomic-image-layout"),
            entries: &[
                texture_entry(12, wgpu::TextureSampleType::Float { filterable: true }),
                sampler_entry(14),
            ],
        });
        let atomic_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-atomic-buffer-layout"),
            entries: &[storage_entry(1, false), storage_entry(3, false)],
        });
        let sampler_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-atomic-sampler-layout"),
            entries: &[sampler_entry(9), sampler_entry(10)],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nuxie-atomic-pipeline-layout"),
            bind_group_layouts: &[
                Some(&flush_layout),
                Some(&image_layout),
                Some(&atomic_layout),
                Some(&sampler_layout),
            ],
            immediate_size: 0,
        });
        let path = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-path-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &path_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(PatchVertex::layout())],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &path_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[
                    ("0", 0.0),
                    ("1", 1.0),
                    ("3", 0.0),
                    ("4", 0.0),
                    ("7", 0.0),
                ]),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::empty(),
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let resolve = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-resolve-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &resolve_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &resolve_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[("0", 0.0), ("1", 1.0), ("4", 0.0), ("7", 0.0)]),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            path,
            resolve,
            flush_layout,
            image_layout,
            atomic_layout,
            sampler_layout,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        patch_vertices: &wgpu::Buffer,
        patch_indices: &wgpu::Buffer,
        tessellation: &wgpu::TextureView,
        uniforms: &FlushUniforms,
        paths: &[PathData],
        paints: &[PaintData],
        paint_aux: &[PaintAuxData],
        contours: &[ContourData],
        base_instance: u32,
        instance_count: u32,
        pixel_count: usize,
    ) {
        let uniform = upload(
            device,
            "nuxie-atomic-uniforms",
            std::slice::from_ref(uniforms),
            wgpu::BufferUsages::UNIFORM,
        );
        let paths = upload(
            device,
            "nuxie-atomic-path-data",
            paths,
            wgpu::BufferUsages::STORAGE,
        );
        let paints = upload(
            device,
            "nuxie-atomic-paint-data",
            paints,
            wgpu::BufferUsages::STORAGE,
        );
        let paint_aux = upload(
            device,
            "nuxie-atomic-paint-aux",
            paint_aux,
            wgpu::BufferUsages::STORAGE,
        );
        let contours = upload(
            device,
            "nuxie-atomic-contours",
            contours,
            wgpu::BufferUsages::STORAGE,
        );
        let colors = upload(
            device,
            "nuxie-atomic-colors",
            &vec![0u32; pixel_count],
            wgpu::BufferUsages::STORAGE,
        );
        let coverage = upload(
            device,
            "nuxie-atomic-coverage",
            &vec![0u32; pixel_count],
            wgpu::BufferUsages::STORAGE,
        );
        let dummy = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-atomic-dummy-texture"),
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
        let sampler = device.create_sampler(&Default::default());
        let flush = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-atomic-flush-group"),
            layout: &self.flush_layout,
            entries: &[
                binding(0, uniform.as_entire_binding()),
                binding(3, paths.as_entire_binding()),
                binding(4, paints.as_entire_binding()),
                binding(5, paint_aux.as_entire_binding()),
                binding(6, contours.as_entire_binding()),
                binding(8, wgpu::BindingResource::TextureView(tessellation)),
                binding(9, wgpu::BindingResource::TextureView(&dummy_view)),
                binding(10, wgpu::BindingResource::TextureView(&dummy_view)),
            ],
        });
        let image = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-atomic-image-group"),
            layout: &self.image_layout,
            entries: &[
                binding(12, wgpu::BindingResource::TextureView(&dummy_view)),
                binding(14, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        let atomics = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-atomic-buffer-group"),
            layout: &self.atomic_layout,
            entries: &[
                binding(1, colors.as_entire_binding()),
                binding(3, coverage.as_entire_binding()),
            ],
        });
        let samplers = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-atomic-sampler-group"),
            layout: &self.sampler_layout,
            entries: &[
                binding(9, wgpu::BindingResource::Sampler(&sampler)),
                binding(10, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        {
            let attachments = [color_attachment(
                target,
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            )];
            let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                "nuxie-atomic-path-pass",
                &attachments,
            ));
            pass.set_pipeline(&self.path);
            pass.set_bind_group(0, &flush, &[]);
            pass.set_bind_group(1, &image, &[]);
            pass.set_bind_group(2, &atomics, &[]);
            pass.set_bind_group(3, &samplers, &[]);
            pass.set_vertex_buffer(0, patch_vertices.slice(..));
            pass.set_index_buffer(patch_indices.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(
                0..crate::gpu::MIDPOINT_FAN_PATCH_INDEX_COUNT as u32,
                0,
                base_instance..base_instance + instance_count,
            );
        }
        {
            let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
            let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                "nuxie-atomic-resolve-pass",
                &attachments,
            ));
            pass.set_pipeline(&self.resolve);
            pass.set_bind_group(0, &flush, &[]);
            pass.set_bind_group(1, &image, &[]);
            pass.set_bind_group(2, &atomics, &[]);
            pass.set_bind_group(3, &samplers, &[]);
            pass.draw(0..4, 0..1);
        }
    }
}

fn shader(device: &wgpu::Device, label: &'static str, source: &'static str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}
fn options(constants: &'static [(&'static str, f64)]) -> wgpu::PipelineCompilationOptions<'static> {
    wgpu::PipelineCompilationOptions {
        constants,
        ..Default::default()
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
fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: if read_only {
            wgpu::ShaderStages::VERTEX_FRAGMENT
        } else {
            wgpu::ShaderStages::FRAGMENT
        },
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
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
fn color_attachment(
    view: &wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
) -> Option<wgpu::RenderPassColorAttachment<'_>> {
    Some(wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load,
            store: wgpu::StoreOp::Store,
        },
    })
}
fn render_pass_descriptor<'a>(
    label: &'static str,
    attachments: &'a [Option<wgpu::RenderPassColorAttachment<'a>>],
) -> wgpu::RenderPassDescriptor<'a> {
    wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: attachments,
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    }
}
