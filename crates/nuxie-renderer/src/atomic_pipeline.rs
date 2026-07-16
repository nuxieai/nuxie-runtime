//! Clockwise-atomic draw and resolve translated from Rive's WebGPU shaders.

use crate::gpu::{
    ImageDrawUniforms, ImageRectVertex, PaintAuxData, PaintData, PatchVertex, TriangleVertex,
};
use crate::tessellator::TessellationFlushResources;
use crate::work_metrics::{CountedCommandEncoderExt, CountedDeviceExt};
use bytemuck::Zeroable;
use nuxie_render_api::{ImageFilter, ImageSampler, ImageWrap};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex, MutexGuard,
};
#[cfg(feature = "perf-diagnostics")]
use std::time::Instant;

const ATOMIC_BUFFER_RING_SIZE: usize = 3;

pub(crate) struct AtomicPipeline {
    path: wgpu::RenderPipeline,
    outer_path: wgpu::RenderPipeline,
    feather_path: wgpu::RenderPipeline,
    feather_stroke_path: wgpu::RenderPipeline,
    stroke_path: wgpu::RenderPipeline,
    interior: wgpu::RenderPipeline,
    atlas_blit: wgpu::RenderPipeline,
    advanced_atlas_blit: wgpu::RenderPipeline,
    advanced_hsl_atlas_blit: wgpu::RenderPipeline,
    image_rect: wgpu::RenderPipeline,
    image_mesh: wgpu::RenderPipeline,
    advanced_path: wgpu::RenderPipeline,
    advanced_outer_path: wgpu::RenderPipeline,
    advanced_feather_path: wgpu::RenderPipeline,
    advanced_feather_hsl_path: wgpu::RenderPipeline,
    advanced_feather_stroke_path: wgpu::RenderPipeline,
    advanced_feather_hsl_stroke_path: wgpu::RenderPipeline,
    advanced_interior: wgpu::RenderPipeline,
    advanced_image_rect: wgpu::RenderPipeline,
    advanced_image_mesh: wgpu::RenderPipeline,
    advanced_init: wgpu::RenderPipeline,
    advanced_resolve: wgpu::RenderPipeline,
    resolve: wgpu::RenderPipeline,
    feather_resolve: wgpu::RenderPipeline,
    flush_layout: wgpu::BindGroupLayout,
    image_layout: wgpu::BindGroupLayout,
    atomic_layout: wgpu::BindGroupLayout,
    sampler_layout: wgpu::BindGroupLayout,
    _dummy_image_texture: wgpu::Texture,
    _dummy_image_view: wgpu::TextureView,
    dummy_image_uniforms: wgpu::Buffer,
    dummy_image_group: wgpu::BindGroup,
    image_samplers: Vec<wgpu::Sampler>,
    image_rect_vertices: wgpu::Buffer,
    image_rect_indices: wgpu::Buffer,
    backing_slots: [Mutex<AtomicBackingSlot>; ATOMIC_BUFFER_RING_SIZE],
    next_backing_slot: AtomicUsize,
}

pub(crate) struct AtomicBackingFrame<'a> {
    slot: MutexGuard<'a, AtomicBackingSlot>,
    #[cfg(feature = "perf-diagnostics")]
    diagnostics: AtomicEncodeDiagnostics,
}

#[cfg(feature = "perf-diagnostics")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct AtomicEncodeDiagnostics {
    pub batches: u64,
    pub draw_groups: u64,
    pub draws: u64,
    pub buffer_upload_ns: u64,
    pub backing_prepare_ns: u64,
    pub dummy_texture_ns: u64,
    pub sampler_create_ns: u64,
    pub flush_bind_groups: u64,
    pub flush_bind_group_ns: u64,
    pub image_bind_groups: u64,
    pub image_bind_group_ns: u64,
    pub load_color_bind_groups: u64,
    pub load_color_bind_group_ns: u64,
    pub atomic_bind_groups: u64,
    pub atomic_bind_group_ns: u64,
    pub sampler_bind_groups: u64,
    pub sampler_bind_group_ns: u64,
    pub render_passes: u64,
    pub render_encode_ns: u64,
    pub total_ns: u64,
}

impl AtomicBackingFrame<'_> {
    #[cfg(feature = "perf-diagnostics")]
    pub(crate) fn diagnostics(&self) -> AtomicEncodeDiagnostics {
        self.diagnostics
    }
}

#[derive(Default)]
struct AtomicBackingSlot {
    colors: Option<AtomicBackingBuffer>,
    clips: Option<AtomicBackingBuffer>,
    coverage: Option<AtomicBackingBuffer>,
}

struct AtomicBackingBuffer {
    buffer: wgpu::Buffer,
    capacity: u64,
}

impl AtomicBackingSlot {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        plane_size: u64,
        color_size: u64,
    ) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
        let colors = backing_buffer(device, &mut self.colors, "nuxie-atomic-colors", color_size);
        let clips = backing_buffer(device, &mut self.clips, "nuxie-atomic-clips", plane_size);
        let coverage = backing_buffer(
            device,
            &mut self.coverage,
            "nuxie-atomic-coverage",
            plane_size,
        );
        encoder.clear_buffer(&colors, 0, Some(color_size));
        encoder.clear_buffer(&clips, 0, Some(plane_size));
        encoder.clear_buffer(&coverage, 0, Some(plane_size));
        (colors, clips, coverage)
    }
}

fn backing_buffer(
    device: &wgpu::Device,
    slot: &mut Option<AtomicBackingBuffer>,
    label: &'static str,
    required_size: u64,
) -> wgpu::Buffer {
    if slot
        .as_ref()
        .is_none_or(|backing| backing.capacity < required_size)
    {
        *slot = Some(AtomicBackingBuffer {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: required_size,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            capacity: required_size,
        });
    }
    slot.as_ref()
        .expect("atomic backing buffer was initialized")
        .buffer
        .clone()
}

pub(crate) struct AtomicDraw<'a> {
    pub tessellation: &'a wgpu::TextureView,
    pub base_instance: u32,
    pub instance_count: u32,
    pub patch_index_range: std::ops::Range<u32>,
    pub triangle_vertices: &'a [crate::gpu::TriangleVertex],
    pub atlas: Option<&'a wgpu::TextureView>,
    pub atlas_blit_vertices: &'a [TriangleVertex],
    pub is_stroke: bool,
    pub is_feather: bool,
    pub batchable_direct_stroke: bool,
    pub hsl_blend: bool,
    pub image: Option<&'a wgpu::TextureView>,
    pub image_sampler: ImageSampler,
    pub image_uniforms: Option<ImageDrawUniforms>,
    pub image_mesh: Option<ImageMeshBuffers<'a>>,
}

pub(crate) struct AtomicPlaneReadback {
    pub buffer: wgpu::Buffer,
    pub word_count: usize,
}

pub(crate) struct AtomicPlaneReadbacks {
    pub coverage: Option<AtomicPlaneReadback>,
    pub clip: Option<AtomicPlaneReadback>,
    pub color: Option<AtomicPlaneReadback>,
}

#[derive(Clone, Copy)]
pub(crate) struct ImageMeshBuffers<'a> {
    pub vertices: &'a wgpu::Buffer,
    pub uvs: &'a wgpu::Buffer,
    pub indices: &'a wgpu::Buffer,
    pub index_count: u32,
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
        let interior_vertex = shader(
            device,
            "nuxie-atomic-interior-vertex",
            include_str!("generated/atomic_draw_interior_triangles.webgpu_vert.wgsl"),
        );
        let interior_fragment = shader(
            device,
            "nuxie-atomic-interior-fragment",
            include_str!("generated/atomic_draw_interior_triangles.webgpu_fixedcolor_frag.wgsl"),
        );
        let atlas_blit_vertex = shader(
            device,
            "nuxie-atomic-atlas-blit-vertex",
            include_str!("generated/atomic_draw_atlas_blit.webgpu_vert.wgsl"),
        );
        let atlas_blit_fragment = shader(
            device,
            "nuxie-atomic-atlas-blit-fragment",
            include_str!("generated/atomic_draw_atlas_blit.webgpu_fixedcolor_frag.wgsl"),
        );
        let advanced_atlas_blit_fragment = shader(
            device,
            "nuxie-atomic-advanced-atlas-blit-fragment",
            include_str!("generated/atomic_draw_atlas_blit.webgpu_frag.wgsl"),
        );
        let image_rect_vertex = shader(
            device,
            "nuxie-atomic-image-rect-vertex",
            include_str!("generated/atomic_draw_image_rect.webgpu_vert.wgsl"),
        );
        let image_rect_fragment = shader(
            device,
            "nuxie-atomic-image-rect-fragment",
            include_str!("generated/atomic_draw_image_rect.webgpu_fixedcolor_frag.wgsl"),
        );
        let image_mesh_vertex = shader(
            device,
            "nuxie-atomic-image-mesh-vertex",
            include_str!("generated/atomic_draw_image_mesh.webgpu_vert.wgsl"),
        );
        let image_mesh_fragment = shader(
            device,
            "nuxie-atomic-image-mesh-fragment",
            include_str!("generated/atomic_draw_image_mesh.webgpu_fixedcolor_frag.wgsl"),
        );
        let advanced_path_fragment = shader(
            device,
            "nuxie-atomic-advanced-path-fragment",
            include_str!("generated/atomic_draw_path.webgpu_frag.wgsl"),
        );
        let advanced_interior_fragment = shader(
            device,
            "nuxie-atomic-advanced-interior-fragment",
            include_str!("generated/atomic_draw_interior_triangles.webgpu_frag.wgsl"),
        );
        let advanced_image_mesh_fragment = shader(
            device,
            "nuxie-atomic-advanced-image-mesh-fragment",
            include_str!("generated/atomic_draw_image_mesh.webgpu_frag.wgsl"),
        );
        let advanced_image_rect_fragment = shader(
            device,
            "nuxie-atomic-advanced-image-rect-fragment",
            include_str!("generated/atomic_draw_image_rect.webgpu_frag.wgsl"),
        );
        let advanced_init_vertex = shader(
            device,
            "nuxie-atomic-advanced-init-vertex",
            include_str!("generated/atomic_init.webgpu_vert.wgsl"),
        );
        let advanced_init_fragment = shader(
            device,
            "nuxie-atomic-advanced-init-fragment",
            include_str!("generated/atomic_init.webgpu_frag.wgsl"),
        );
        let advanced_resolve_vertex = shader(
            device,
            "nuxie-atomic-advanced-resolve-vertex",
            include_str!("generated/atomic_resolve_coalesced.webgpu_vert.wgsl"),
        );
        let advanced_resolve_fragment = shader(
            device,
            "nuxie-atomic-advanced-resolve-fragment",
            include_str!("generated/atomic_resolve_coalesced.webgpu_frag.wgsl"),
        );
        let flush_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-atomic-flush-layout"),
            entries: &[
                uniform_entry(0),
                uniform_entry(2),
                storage_entry(3, true),
                storage_entry(4, true),
                storage_entry(5, true),
                storage_entry(6, true),
                texture_entry(8, wgpu::TextureSampleType::Uint),
                texture_entry(9, wgpu::TextureSampleType::Float { filterable: true }),
                texture_entry(10, wgpu::TextureSampleType::Float { filterable: true }),
                texture_entry(11, wgpu::TextureSampleType::Float { filterable: true }),
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
            entries: &[
                storage_entry(0, false),
                storage_entry(1, false),
                storage_entry(3, false),
            ],
        });
        let sampler_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-atomic-sampler-layout"),
            entries: &[sampler_entry(9), sampler_entry(10), sampler_entry(11)],
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
        let image_rect = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-image-rect-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &image_rect_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(ImageRectVertex::layout())],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &image_rect_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[("0", 1.0), ("1", 1.0), ("4", 0.0), ("7", 0.0)]),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let image_mesh = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-image-mesh-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &image_mesh_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[
                    Some(image_mesh_vertex_layout(0)),
                    Some(image_mesh_vertex_layout(1)),
                ],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &image_mesh_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[("0", 1.0), ("1", 1.0), ("4", 0.0), ("7", 0.0)]),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let make_advanced_path = |label, cull_mode, feather, hsl| {
            let constants = advanced_path_constants(feather, hsl);
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &path_vertex,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(PatchVertex::layout())],
                },
                primitive: wgpu::PrimitiveState {
                    cull_mode,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &advanced_path_fragment,
                    entry_point: Some("main"),
                    compilation_options: options(&constants),
                    targets: &[Some(disabled_color_target())],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let advanced_path = make_advanced_path(
            "nuxie-atomic-advanced-path-pipeline",
            Some(wgpu::Face::Front),
            0.0,
            1.0,
        );
        let advanced_outer_path = make_advanced_path(
            "nuxie-atomic-advanced-outer-path-pipeline",
            // C++ WebGPU declares CW front and culls Back (CCW). wgpu's
            // default front face is CCW, so its equivalent is culling Front.
            Some(wgpu::Face::Front),
            0.0,
            1.0,
        );
        let advanced_feather_path = make_advanced_path(
            "nuxie-atomic-advanced-feather-path-pipeline",
            Some(wgpu::Face::Front),
            1.0,
            0.0,
        );
        let advanced_feather_hsl_path = make_advanced_path(
            "nuxie-atomic-advanced-feather-hsl-path-pipeline",
            Some(wgpu::Face::Front),
            1.0,
            1.0,
        );
        let advanced_feather_stroke_path = make_advanced_path(
            "nuxie-atomic-advanced-feather-stroke-path-pipeline",
            Some(wgpu::Face::Front),
            1.0,
            0.0,
        );
        let advanced_feather_hsl_stroke_path = make_advanced_path(
            "nuxie-atomic-advanced-feather-hsl-stroke-path-pipeline",
            Some(wgpu::Face::Front),
            1.0,
            1.0,
        );
        let advanced_hsl_fill_constants = advanced_fill_constants(1.0);
        let advanced_interior = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-advanced-interior-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &interior_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(crate::gpu::TriangleVertex::layout())],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &advanced_interior_fragment,
                entry_point: Some("main"),
                compilation_options: options(&advanced_hsl_fill_constants),
                targets: &[Some(disabled_color_target())],
            }),
            multiview_mask: None,
            cache: None,
        });
        let advanced_image_mesh = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-advanced-image-mesh-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &image_mesh_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[
                    Some(image_mesh_vertex_layout(0)),
                    Some(image_mesh_vertex_layout(1)),
                ],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &advanced_image_mesh_fragment,
                entry_point: Some("main"),
                compilation_options: options(&advanced_hsl_fill_constants),
                targets: &[Some(disabled_color_target())],
            }),
            multiview_mask: None,
            cache: None,
        });
        let advanced_image_rect = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-advanced-image-rect-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &image_rect_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(ImageRectVertex::layout())],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &advanced_image_rect_fragment,
                entry_point: Some("main"),
                compilation_options: options(&advanced_hsl_fill_constants),
                targets: &[Some(disabled_color_target())],
            }),
            multiview_mask: None,
            cache: None,
        });
        let advanced_init = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-advanced-init-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &advanced_init_vertex,
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
                module: &advanced_init_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[("0", 1.0), ("11", 0.0), ("12", 1.0)]),
                targets: &[Some(disabled_color_target())],
            }),
            multiview_mask: None,
            cache: None,
        });
        let advanced_resolve = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-advanced-resolve-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &advanced_resolve_vertex,
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
                module: &advanced_resolve_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[
                    ("0", 1.0),
                    ("1", 1.0),
                    ("2", 1.0),
                    ("4", 0.0),
                    ("6", 1.0),
                ]),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let dummy_image_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-atomic-dummy-image"),
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
        let dummy_image_view = dummy_image_texture.create_view(&Default::default());
        let dummy_image_uniforms = upload(
            device,
            "nuxie-atomic-dummy-image-uniforms",
            &[ImageDrawUniforms::zeroed()],
            wgpu::BufferUsages::UNIFORM,
        );
        let image_samplers = [ImageFilter::Bilinear, ImageFilter::Nearest]
            .into_iter()
            .flat_map(|filter| {
                [ImageWrap::Clamp, ImageWrap::Repeat, ImageWrap::Mirror]
                    .into_iter()
                    .flat_map(move |wrap_y| {
                        [ImageWrap::Clamp, ImageWrap::Repeat, ImageWrap::Mirror]
                            .into_iter()
                            .map(move |wrap_x| ImageSampler {
                                wrap_x,
                                wrap_y,
                                filter,
                            })
                    })
            })
            .map(|sampler| device.create_sampler(&image_sampler(sampler)))
            .collect::<Vec<_>>();
        debug_assert_eq!(image_samplers.len(), 18);
        let dummy_image_group = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-atomic-dummy-image-group"),
            layout: &image_layout,
            entries: &[
                binding(12, wgpu::BindingResource::TextureView(&dummy_image_view)),
                binding(14, wgpu::BindingResource::Sampler(&image_samplers[0])),
            ],
        });
        let image_rect_vertices = upload(
            device,
            "nuxie-atomic-image-rect-vertices",
            &crate::gpu::IMAGE_RECT_VERTICES,
            wgpu::BufferUsages::VERTEX,
        );
        let image_rect_indices = upload(
            device,
            "nuxie-atomic-image-rect-indices",
            &crate::gpu::IMAGE_RECT_INDICES,
            wgpu::BufferUsages::INDEX,
        );
        let path = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-path-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &path_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(PatchVertex::layout())],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &path_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[
                    ("0", 1.0),
                    ("1", 1.0),
                    ("3", 0.0),
                    ("4", 0.0),
                    ("7", 0.0),
                ]),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let outer_path = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-outer-path-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &path_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(PatchVertex::layout())],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &path_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[
                    ("0", 1.0),
                    ("1", 1.0),
                    ("3", 0.0),
                    ("4", 0.0),
                    ("7", 0.0),
                ]),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let feather_path = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-feather-path-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &path_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(PatchVertex::layout())],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &path_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[
                    ("0", 1.0),
                    ("1", 1.0),
                    ("3", 1.0),
                    ("4", 0.0),
                    ("7", 1.0),
                ]),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let feather_stroke_path = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-feather-stroke-path-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &path_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(PatchVertex::layout())],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &path_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[
                    ("0", 1.0),
                    ("1", 1.0),
                    ("3", 1.0),
                    ("4", 0.0),
                    ("7", 1.0),
                ]),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let stroke_path = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-stroke-path-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &path_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(PatchVertex::layout())],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &path_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[
                    ("0", 1.0),
                    ("1", 1.0),
                    ("3", 0.0),
                    ("4", 0.0),
                    ("7", 0.0),
                ]),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let make_resolve = |label, dither| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
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
                    compilation_options: options(&[
                        ("0", 1.0),
                        ("1", 1.0),
                        ("4", 0.0),
                        ("7", dither),
                    ]),
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
        let resolve = make_resolve("nuxie-atomic-resolve-pipeline", 0.0);
        let feather_resolve = make_resolve("nuxie-atomic-feather-resolve-pipeline", 1.0);
        let interior = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-interior-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &interior_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(crate::gpu::TriangleVertex::layout())],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &interior_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[("0", 1.0), ("1", 1.0), ("4", 0.0), ("7", 0.0)]),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let atlas_blit = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-atomic-atlas-blit-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &atlas_blit_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(TriangleVertex::layout())],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &atlas_blit_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[("0", 1.0), ("1", 1.0), ("4", 0.0), ("7", 1.0)]),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let make_advanced_atlas_blit = |label, hsl| {
            let constants = advanced_fill_constants(hsl);
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &atlas_blit_vertex,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(TriangleVertex::layout())],
                },
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &advanced_atlas_blit_fragment,
                    entry_point: Some("main"),
                    compilation_options: options(&constants),
                    targets: &[Some(disabled_color_target())],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let advanced_atlas_blit =
            make_advanced_atlas_blit("nuxie-atomic-advanced-atlas-blit-pipeline", 0.0);
        let advanced_hsl_atlas_blit =
            make_advanced_atlas_blit("nuxie-atomic-advanced-hsl-atlas-blit-pipeline", 1.0);
        Self {
            path,
            outer_path,
            feather_path,
            feather_stroke_path,
            stroke_path,
            interior,
            atlas_blit,
            advanced_atlas_blit,
            advanced_hsl_atlas_blit,
            image_rect,
            image_mesh,
            advanced_path,
            advanced_outer_path,
            advanced_feather_path,
            advanced_feather_hsl_path,
            advanced_feather_stroke_path,
            advanced_feather_hsl_stroke_path,
            advanced_interior,
            advanced_image_rect,
            advanced_image_mesh,
            advanced_init,
            advanced_resolve,
            resolve,
            feather_resolve,
            flush_layout,
            image_layout,
            atomic_layout,
            sampler_layout,
            _dummy_image_texture: dummy_image_texture,
            _dummy_image_view: dummy_image_view,
            dummy_image_uniforms,
            dummy_image_group,
            image_samplers,
            image_rect_vertices,
            image_rect_indices,
            backing_slots: std::array::from_fn(|_| Mutex::new(AtomicBackingSlot::default())),
            next_backing_slot: AtomicUsize::new(0),
        }
    }

    pub(crate) fn begin_frame_backing(&self) -> AtomicBackingFrame<'_> {
        let slot_index =
            self.next_backing_slot.fetch_add(1, Ordering::Relaxed) % ATOMIC_BUFFER_RING_SIZE;
        let slot = self.backing_slots[slot_index]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        AtomicBackingFrame {
            slot,
            #[cfg(feature = "perf-diagnostics")]
            diagnostics: AtomicEncodeDiagnostics::default(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_batch(
        &self,
        device: &wgpu::Device,
        backing: &mut AtomicBackingFrame<'_>,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        load_color: Option<&wgpu::TextureView>,
        feather_lut: &wgpu::TextureView,
        gradient: Option<&wgpu::TextureView>,
        patch_vertices: &wgpu::Buffer,
        patch_indices: &wgpu::Buffer,
        draws: &[AtomicDraw<'_>],
        draw_group_starts: &[usize],
        batch_shared_draws: bool,
        flush_resources: &TessellationFlushResources,
        paints: &[PaintData],
        paint_aux: &[PaintAuxData],
        pixel_count: usize,
        capture_planes: bool,
    ) -> AtomicPlaneReadbacks {
        assert!(!draws.is_empty());
        assert_eq!(draw_group_starts.first(), Some(&0));
        assert!(draw_group_starts
            .windows(2)
            .all(|starts| starts[0] < starts[1]));
        assert!(draw_group_starts
            .last()
            .is_some_and(|start| *start < draws.len()));
        #[cfg(feature = "perf-diagnostics")]
        let total_started = Instant::now();
        #[cfg(feature = "perf-diagnostics")]
        {
            backing.diagnostics.batches = backing.diagnostics.batches.saturating_add(1);
            backing.diagnostics.draw_groups = backing
                .diagnostics
                .draw_groups
                .saturating_add(draw_group_starts.len() as u64);
            backing.diagnostics.draws =
                backing.diagnostics.draws.saturating_add(draws.len() as u64);
        }
        // C++ RenderContextWebGPUImpl::AtomicDrawRenderPass switches the whole
        // flush to storage-buffer color when fixedFunctionColorOutput is false.
        let advanced_blend = paints.iter().any(|paint| (paint.params >> 4) & 0xf != 0)
            || draws.iter().any(|draw| {
                draw.image_uniforms
                    .is_some_and(|uniforms| uniforms.blend_mode != 0)
            });
        // Each draw first resolves coverage left by the previous draw. Shader
        // features therefore describe the whole batch, not only the geometry
        // currently being emitted.
        let hsl_blend = draws.iter().any(|draw| draw.hsl_blend);
        #[cfg(feature = "perf-diagnostics")]
        let buffer_upload_started = Instant::now();
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
        #[cfg(feature = "perf-diagnostics")]
        {
            backing.diagnostics.buffer_upload_ns = backing
                .diagnostics
                .buffer_upload_ns
                .saturating_add(elapsed_ns(buffer_upload_started));
        }
        let plane_size = u64::try_from(pixel_count)
            .expect("atomic backing word count fits u64")
            .checked_mul(std::mem::size_of::<u32>() as u64)
            .expect("atomic backing byte size overflow");
        let color_word_count = if advanced_blend { pixel_count } else { 1 };
        let color_size = u64::try_from(color_word_count)
            .expect("atomic color word count fits u64")
            .checked_mul(std::mem::size_of::<u32>() as u64)
            .expect("atomic color byte size overflow");
        #[cfg(feature = "perf-diagnostics")]
        let backing_prepare_started = Instant::now();
        let (colors, clips, coverage) = backing
            .slot
            .prepare(device, encoder, plane_size, color_size);
        #[cfg(feature = "perf-diagnostics")]
        {
            backing.diagnostics.backing_prepare_ns = backing
                .diagnostics
                .backing_prepare_ns
                .saturating_add(elapsed_ns(backing_prepare_started));
        }
        let clip_readback = capture_planes.then(|| AtomicPlaneReadback {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("nuxie-atomic-clip-readback"),
                size: plane_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            word_count: pixel_count,
        });
        let coverage_readback = capture_planes.then(|| AtomicPlaneReadback {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("nuxie-atomic-coverage-readback"),
                size: plane_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            word_count: pixel_count,
        });
        let color_readback = (capture_planes && advanced_blend).then(|| AtomicPlaneReadback {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("nuxie-atomic-color-readback"),
                size: color_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            word_count: color_word_count,
        });
        let shared_flush_group = draws.iter().all(|draw| {
            std::ptr::eq(draw.tessellation, draws[0].tessellation)
                && draw.atlas.is_none()
                && draws[0].atlas.is_none()
                && draw.image.is_none()
        });
        #[cfg(feature = "perf-diagnostics")]
        let triangle_upload_started = Instant::now();
        let (shared_triangle_buffer, shared_triangle_ranges, triangle_buffers) =
            if shared_flush_group {
                let mut vertices = Vec::new();
                let ranges = draws
                    .iter()
                    .map(|draw| {
                        let start = vertices.len() as u32;
                        vertices.extend_from_slice(draw.triangle_vertices);
                        start..vertices.len() as u32
                    })
                    .collect::<Vec<_>>();
                let buffer = (!vertices.is_empty()).then(|| {
                    upload(
                        device,
                        "nuxie-atomic-shared-triangles",
                        &vertices,
                        wgpu::BufferUsages::VERTEX,
                    )
                });
                (buffer, ranges, Vec::new())
            } else {
                let buffers = draws
                    .iter()
                    .map(|draw| {
                        let vertices = if draw.atlas.is_some() {
                            draw.atlas_blit_vertices
                        } else {
                            draw.triangle_vertices
                        };
                        (!vertices.is_empty()).then(|| {
                            upload(
                                device,
                                "nuxie-atomic-triangles",
                                vertices,
                                wgpu::BufferUsages::VERTEX,
                            )
                        })
                    })
                    .collect::<Vec<_>>();
                (None, Vec::new(), buffers)
            };
        #[cfg(feature = "perf-diagnostics")]
        {
            backing.diagnostics.buffer_upload_ns = backing
                .diagnostics
                .buffer_upload_ns
                .saturating_add(elapsed_ns(triangle_upload_started));
        }
        #[cfg(feature = "perf-diagnostics")]
        let dummy_texture_started = Instant::now();
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
        #[cfg(feature = "perf-diagnostics")]
        {
            backing.diagnostics.dummy_texture_ns = backing
                .diagnostics
                .dummy_texture_ns
                .saturating_add(elapsed_ns(dummy_texture_started));
        }
        #[cfg(feature = "perf-diagnostics")]
        let sampler_create_started = Instant::now();
        let sampler = device.create_sampler(&linear_sampler());
        #[cfg(feature = "perf-diagnostics")]
        {
            backing.diagnostics.sampler_create_ns = backing
                .diagnostics
                .sampler_create_ns
                .saturating_add(elapsed_ns(sampler_create_started));
        }
        #[cfg(feature = "perf-diagnostics")]
        let image_uniform_upload_started = Instant::now();
        let image_uniform_buffers = draws
            .iter()
            .map(|draw| {
                draw.image_uniforms.map(|uniforms| {
                    upload(
                        device,
                        "nuxie-atomic-image-uniforms",
                        std::slice::from_ref(&uniforms),
                        wgpu::BufferUsages::UNIFORM,
                    )
                })
            })
            .collect::<Vec<_>>();
        #[cfg(feature = "perf-diagnostics")]
        {
            backing.diagnostics.buffer_upload_ns = backing
                .diagnostics
                .buffer_upload_ns
                .saturating_add(elapsed_ns(image_uniform_upload_started));
        }
        let make_flush_group = |draw_index: usize, draw: &AtomicDraw<'_>| {
            device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("nuxie-atomic-flush-group"),
                layout: &self.flush_layout,
                entries: &[
                    binding(0, flush_resources.uniform_binding()),
                    binding(
                        2,
                        image_uniform_buffers[draw_index]
                            .as_ref()
                            .unwrap_or(&self.dummy_image_uniforms)
                            .as_entire_binding(),
                    ),
                    binding(3, flush_resources.path_binding()),
                    binding(4, paints.as_entire_binding()),
                    binding(5, paint_aux.as_entire_binding()),
                    binding(6, flush_resources.contour_binding()),
                    binding(8, wgpu::BindingResource::TextureView(draw.tessellation)),
                    binding(
                        9,
                        wgpu::BindingResource::TextureView(gradient.unwrap_or(&dummy_view)),
                    ),
                    binding(10, wgpu::BindingResource::TextureView(feather_lut)),
                    binding(
                        11,
                        wgpu::BindingResource::TextureView(draw.atlas.unwrap_or(&dummy_view)),
                    ),
                ],
            })
        };
        #[cfg(feature = "perf-diagnostics")]
        let flush_bind_group_started = Instant::now();
        let flush_groups = if shared_flush_group {
            vec![make_flush_group(0, &draws[0])]
        } else {
            draws
                .iter()
                .enumerate()
                .map(|(index, draw)| make_flush_group(index, draw))
                .collect::<Vec<_>>()
        };
        #[cfg(feature = "perf-diagnostics")]
        {
            backing.diagnostics.flush_bind_groups = backing
                .diagnostics
                .flush_bind_groups
                .saturating_add(flush_groups.len() as u64);
            backing.diagnostics.flush_bind_group_ns = backing
                .diagnostics
                .flush_bind_group_ns
                .saturating_add(elapsed_ns(flush_bind_group_started));
        }
        let flush_group_index = |draw_index: usize| if shared_flush_group { 0 } else { draw_index };
        #[cfg(feature = "perf-diagnostics")]
        let image_bind_group_started = Instant::now();
        let image_groups = draws
            .iter()
            .map(|draw| {
                draw.image.map(|image| {
                    device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("nuxie-atomic-image-group"),
                        layout: &self.image_layout,
                        entries: &[
                            binding(12, wgpu::BindingResource::TextureView(image)),
                            binding(
                                14,
                                wgpu::BindingResource::Sampler(
                                    &self.image_samplers[draw.image_sampler.as_key() as usize],
                                ),
                            ),
                        ],
                    })
                })
            })
            .collect::<Vec<_>>();
        #[cfg(feature = "perf-diagnostics")]
        {
            backing.diagnostics.image_bind_groups = backing
                .diagnostics
                .image_bind_groups
                .saturating_add(image_groups.iter().flatten().count() as u64);
            backing.diagnostics.image_bind_group_ns = backing
                .diagnostics
                .image_bind_group_ns
                .saturating_add(elapsed_ns(image_bind_group_started));
        }
        let image_group = |draw_index: usize| {
            image_groups[draw_index]
                .as_ref()
                .unwrap_or(&self.dummy_image_group)
        };
        #[cfg(feature = "perf-diagnostics")]
        let load_color_bind_group_started = Instant::now();
        let load_color_group = load_color.map(|view| {
            device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("nuxie-atomic-load-color-group"),
                layout: &self.image_layout,
                entries: &[
                    binding(12, wgpu::BindingResource::TextureView(view)),
                    binding(14, wgpu::BindingResource::Sampler(&self.image_samplers[0])),
                ],
            })
        });
        #[cfg(feature = "perf-diagnostics")]
        {
            backing.diagnostics.load_color_bind_groups = backing
                .diagnostics
                .load_color_bind_groups
                .saturating_add(u64::from(load_color_group.is_some()));
            backing.diagnostics.load_color_bind_group_ns = backing
                .diagnostics
                .load_color_bind_group_ns
                .saturating_add(elapsed_ns(load_color_bind_group_started));
        }
        #[cfg(feature = "perf-diagnostics")]
        let atomic_bind_group_started = Instant::now();
        let atomics = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-atomic-buffer-group"),
            layout: &self.atomic_layout,
            entries: &[
                binding(0, colors.as_entire_binding()),
                binding(1, clips.as_entire_binding()),
                binding(3, coverage.as_entire_binding()),
            ],
        });
        #[cfg(feature = "perf-diagnostics")]
        {
            backing.diagnostics.atomic_bind_groups =
                backing.diagnostics.atomic_bind_groups.saturating_add(1);
            backing.diagnostics.atomic_bind_group_ns = backing
                .diagnostics
                .atomic_bind_group_ns
                .saturating_add(elapsed_ns(atomic_bind_group_started));
        }
        #[cfg(feature = "perf-diagnostics")]
        let sampler_bind_group_started = Instant::now();
        let samplers = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-atomic-sampler-group"),
            layout: &self.sampler_layout,
            entries: &[
                binding(9, wgpu::BindingResource::Sampler(&sampler)),
                binding(10, wgpu::BindingResource::Sampler(&sampler)),
                binding(11, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        #[cfg(feature = "perf-diagnostics")]
        {
            backing.diagnostics.sampler_bind_groups =
                backing.diagnostics.sampler_bind_groups.saturating_add(1);
            backing.diagnostics.sampler_bind_group_ns = backing
                .diagnostics
                .sampler_bind_group_ns
                .saturating_add(elapsed_ns(sampler_bind_group_started));
        }
        #[cfg(feature = "perf-diagnostics")]
        let render_encode_started = Instant::now();
        if advanced_blend {
            let load_color_group = load_color_group
                .as_ref()
                .expect("advanced atomic blending requires a destination-color copy");
            let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
            #[cfg(feature = "perf-diagnostics")]
            {
                backing.diagnostics.render_passes =
                    backing.diagnostics.render_passes.saturating_add(1);
            }
            let mut pass = encoder.begin_counted_render_pass(&render_pass_descriptor(
                "nuxie-atomic-advanced-init-pass",
                &attachments,
            ));
            pass.set_pipeline(&self.advanced_init);
            pass.set_bind_group(0, &flush_groups[0], &[]);
            pass.set_bind_group(1, load_color_group, &[]);
            pass.set_bind_group(2, &atomics, &[]);
            pass.set_bind_group(3, &samplers, &[]);
            pass.draw(0..4, 0..1);
        }
        if batch_shared_draws && shared_flush_group && draws.iter().all(|draw| draw.atlas.is_none())
        {
            for (group_index, &group_start) in draw_group_starts.iter().enumerate() {
                let group_end = draw_group_starts
                    .get(group_index + 1)
                    .copied()
                    .unwrap_or(draws.len());
                let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
                #[cfg(feature = "perf-diagnostics")]
                {
                    backing.diagnostics.render_passes =
                        backing.diagnostics.render_passes.saturating_add(1);
                }
                let mut pass = encoder.begin_counted_render_pass(&render_pass_descriptor(
                    "nuxie-atomic-path-pass",
                    &attachments,
                ));
                pass.set_bind_group(1, image_group(0), &[]);
                pass.set_bind_group(2, &atomics, &[]);
                pass.set_bind_group(3, &samplers, &[]);
                pass.set_vertex_buffer(0, patch_vertices.slice(..));
                pass.set_index_buffer(patch_indices.slice(..), wgpu::IndexFormat::Uint16);
                let group_draws = &draws[group_start..group_end];
                let first_draw = &group_draws[0];
                let batch_paths = (shared_triangle_buffer.is_some()
                    || group_draws.iter().all(|draw| draw.batchable_direct_stroke))
                    && group_draws.iter().all(|draw| {
                        draw.patch_index_range == first_draw.patch_index_range
                            && draw.is_feather == first_draw.is_feather
                            && draw.is_stroke == first_draw.is_stroke
                            && draw.triangle_vertices.is_empty()
                                == first_draw.triangle_vertices.is_empty()
                    })
                    && group_draws.windows(2).all(|pair| {
                        pair[0].base_instance + pair[0].instance_count == pair[1].base_instance
                    });
                let path_pipeline = |draw: &AtomicDraw<'_>| {
                    if advanced_blend && draw.is_feather && draw.is_stroke && hsl_blend {
                        &self.advanced_feather_hsl_stroke_path
                    } else if advanced_blend && draw.is_feather && draw.is_stroke {
                        &self.advanced_feather_stroke_path
                    } else if advanced_blend && draw.is_feather && hsl_blend {
                        &self.advanced_feather_hsl_path
                    } else if advanced_blend && draw.is_feather {
                        &self.advanced_feather_path
                    } else if advanced_blend && !draw.triangle_vertices.is_empty() {
                        &self.advanced_outer_path
                    } else if advanced_blend {
                        &self.advanced_path
                    } else if draw.is_feather && draw.is_stroke {
                        &self.feather_stroke_path
                    } else if draw.is_feather {
                        &self.feather_path
                    } else if draw.is_stroke {
                        &self.stroke_path
                    } else if !draw.triangle_vertices.is_empty() {
                        &self.outer_path
                    } else {
                        &self.path
                    }
                };
                if batch_paths {
                    let last_draw = group_draws.last().expect("atomic draw group is nonempty");
                    pass.set_pipeline(path_pipeline(first_draw));
                    pass.set_bind_group(0, &flush_groups[0], &[]);
                    pass.draw_path_patches(
                        first_draw.patch_index_range.clone(),
                        0,
                        first_draw.base_instance
                            ..last_draw.base_instance + last_draw.instance_count,
                    );
                } else {
                    for (draw_index, draw) in group_draws.iter().enumerate() {
                        let draw_index = group_start + draw_index;
                        pass.set_pipeline(path_pipeline(draw));
                        pass.set_bind_group(0, &flush_groups[flush_group_index(draw_index)], &[]);
                        pass.draw_path_patches(
                            draw.patch_index_range.clone(),
                            0,
                            draw.base_instance..draw.base_instance + draw.instance_count,
                        );
                    }
                }
                drop(pass);
                if let Some(triangle_buffer) = &shared_triangle_buffer {
                    let vertex_range = shared_triangle_ranges[group_start].start
                        ..shared_triangle_ranges[group_end - 1].end;
                    if !vertex_range.is_empty() {
                        #[cfg(feature = "perf-diagnostics")]
                        {
                            backing.diagnostics.render_passes =
                                backing.diagnostics.render_passes.saturating_add(1);
                        }
                        let mut interior_pass = encoder.begin_counted_render_pass(
                            &render_pass_descriptor("nuxie-atomic-interior-pass", &attachments),
                        );
                        interior_pass.set_pipeline(if advanced_blend {
                            &self.advanced_interior
                        } else {
                            &self.interior
                        });
                        interior_pass.set_bind_group(0, &flush_groups[0], &[]);
                        interior_pass.set_bind_group(1, image_group(0), &[]);
                        interior_pass.set_bind_group(2, &atomics, &[]);
                        interior_pass.set_bind_group(3, &samplers, &[]);
                        interior_pass.set_vertex_buffer(0, triangle_buffer.slice(..));
                        interior_pass.draw(vertex_range, 0..1);
                    }
                }
            }
        } else {
            for (draw_index, draw) in draws.iter().enumerate() {
                if draw.atlas.is_some() {
                    let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
                    #[cfg(feature = "perf-diagnostics")]
                    {
                        backing.diagnostics.render_passes =
                            backing.diagnostics.render_passes.saturating_add(1);
                    }
                    let mut pass = encoder.begin_counted_render_pass(&render_pass_descriptor(
                        "nuxie-atomic-atlas-blit-pass",
                        &attachments,
                    ));
                    pass.set_pipeline(if advanced_blend && hsl_blend {
                        &self.advanced_hsl_atlas_blit
                    } else if advanced_blend {
                        &self.advanced_atlas_blit
                    } else {
                        &self.atlas_blit
                    });
                    pass.set_bind_group(0, &flush_groups[flush_group_index(draw_index)], &[]);
                    pass.set_bind_group(1, image_group(draw_index), &[]);
                    pass.set_bind_group(2, &atomics, &[]);
                    pass.set_bind_group(3, &samplers, &[]);
                    pass.set_vertex_buffer(
                        0,
                        triangle_buffers[draw_index].as_ref().unwrap().slice(..),
                    );
                    pass.draw(0..draw.atlas_blit_vertices.len() as u32, 0..1);
                    continue;
                }
                if let Some(mesh) = draw.image_mesh {
                    let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
                    #[cfg(feature = "perf-diagnostics")]
                    {
                        backing.diagnostics.render_passes =
                            backing.diagnostics.render_passes.saturating_add(1);
                    }
                    let mut pass = encoder.begin_counted_render_pass(&render_pass_descriptor(
                        "nuxie-atomic-image-mesh-pass",
                        &attachments,
                    ));
                    pass.set_pipeline(if advanced_blend {
                        &self.advanced_image_mesh
                    } else {
                        &self.image_mesh
                    });
                    pass.set_bind_group(0, &flush_groups[flush_group_index(draw_index)], &[]);
                    pass.set_bind_group(1, image_group(draw_index), &[]);
                    pass.set_bind_group(2, &atomics, &[]);
                    pass.set_bind_group(3, &samplers, &[]);
                    pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                    pass.set_vertex_buffer(1, mesh.uvs.slice(..));
                    pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint16);
                    pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    continue;
                }
                if draw.image.is_some() {
                    let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
                    #[cfg(feature = "perf-diagnostics")]
                    {
                        backing.diagnostics.render_passes =
                            backing.diagnostics.render_passes.saturating_add(1);
                    }
                    let mut pass = encoder.begin_counted_render_pass(&render_pass_descriptor(
                        "nuxie-atomic-image-rect-pass",
                        &attachments,
                    ));
                    pass.set_pipeline(if advanced_blend {
                        &self.advanced_image_rect
                    } else {
                        &self.image_rect
                    });
                    pass.set_bind_group(0, &flush_groups[flush_group_index(draw_index)], &[]);
                    pass.set_bind_group(1, image_group(draw_index), &[]);
                    pass.set_bind_group(2, &atomics, &[]);
                    pass.set_bind_group(3, &samplers, &[]);
                    pass.set_vertex_buffer(0, self.image_rect_vertices.slice(..));
                    pass.set_index_buffer(
                        self.image_rect_indices.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );
                    pass.draw_indexed(0..crate::gpu::IMAGE_RECT_INDICES.len() as u32, 0, 0..1);
                    continue;
                }
                let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
                #[cfg(feature = "perf-diagnostics")]
                {
                    backing.diagnostics.render_passes =
                        backing.diagnostics.render_passes.saturating_add(1);
                }
                let mut pass = encoder.begin_counted_render_pass(&render_pass_descriptor(
                    "nuxie-atomic-path-pass",
                    &attachments,
                ));
                pass.set_pipeline(
                    if advanced_blend && draw.is_feather && draw.is_stroke && hsl_blend {
                        &self.advanced_feather_hsl_stroke_path
                    } else if advanced_blend && draw.is_feather && draw.is_stroke {
                        &self.advanced_feather_stroke_path
                    } else if advanced_blend && draw.is_feather && hsl_blend {
                        &self.advanced_feather_hsl_path
                    } else if advanced_blend && draw.is_feather {
                        &self.advanced_feather_path
                    } else if advanced_blend && !draw.triangle_vertices.is_empty() {
                        &self.advanced_outer_path
                    } else if advanced_blend {
                        &self.advanced_path
                    } else if draw.is_feather && draw.is_stroke {
                        &self.feather_stroke_path
                    } else if draw.is_feather {
                        &self.feather_path
                    } else if draw.is_stroke {
                        &self.stroke_path
                    } else if !draw.triangle_vertices.is_empty() {
                        &self.outer_path
                    } else {
                        &self.path
                    },
                );
                pass.set_bind_group(0, &flush_groups[flush_group_index(draw_index)], &[]);
                pass.set_bind_group(1, image_group(draw_index), &[]);
                pass.set_bind_group(2, &atomics, &[]);
                pass.set_bind_group(3, &samplers, &[]);
                pass.set_vertex_buffer(0, patch_vertices.slice(..));
                pass.set_index_buffer(patch_indices.slice(..), wgpu::IndexFormat::Uint16);
                pass.draw_path_patches(
                    draw.patch_index_range.clone(),
                    0,
                    draw.base_instance..draw.base_instance + draw.instance_count,
                );
                drop(pass);
                let triangle_buffer = triangle_vertex_range(
                    shared_flush_group,
                    &shared_triangle_ranges,
                    draw_index,
                    draw.triangle_vertices.len() as u32,
                )
                .and_then(|range| {
                    if shared_flush_group {
                        shared_triangle_buffer
                            .as_ref()
                            .map(|buffer| (buffer, range))
                    } else {
                        triangle_buffers[draw_index]
                            .as_ref()
                            .map(|buffer| (buffer, range))
                    }
                });
                if let Some((triangle_buffer, vertex_range)) = triangle_buffer {
                    let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
                    #[cfg(feature = "perf-diagnostics")]
                    {
                        backing.diagnostics.render_passes =
                            backing.diagnostics.render_passes.saturating_add(1);
                    }
                    let mut pass = encoder.begin_counted_render_pass(&render_pass_descriptor(
                        "nuxie-atomic-interior-pass",
                        &attachments,
                    ));
                    pass.set_pipeline(if advanced_blend {
                        &self.advanced_interior
                    } else {
                        &self.interior
                    });
                    pass.set_bind_group(0, &flush_groups[flush_group_index(draw_index)], &[]);
                    pass.set_bind_group(1, image_group(draw_index), &[]);
                    pass.set_bind_group(2, &atomics, &[]);
                    pass.set_bind_group(3, &samplers, &[]);
                    pass.set_vertex_buffer(0, triangle_buffer.slice(..));
                    pass.draw(vertex_range, 0..1);
                }
            }
        }
        if let Some(readback) = &coverage_readback {
            encoder.copy_buffer_to_buffer(&coverage, 0, &readback.buffer, 0, plane_size);
        }
        if let Some(readback) = &clip_readback {
            encoder.copy_buffer_to_buffer(&clips, 0, &readback.buffer, 0, plane_size);
        }
        if let Some(readback) = &color_readback {
            encoder.copy_buffer_to_buffer(&colors, 0, &readback.buffer, 0, color_size);
        }
        {
            let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
            #[cfg(feature = "perf-diagnostics")]
            {
                backing.diagnostics.render_passes =
                    backing.diagnostics.render_passes.saturating_add(1);
            }
            let mut pass = encoder.begin_counted_render_pass(&render_pass_descriptor(
                "nuxie-atomic-resolve-pass",
                &attachments,
            ));
            pass.set_pipeline(if advanced_blend {
                &self.advanced_resolve
            } else if draws.iter().any(|draw| draw.is_feather) {
                &self.feather_resolve
            } else {
                &self.resolve
            });
            pass.set_bind_group(0, &flush_groups[flush_group_index(draws.len() - 1)], &[]);
            pass.set_bind_group(1, image_group(draws.len() - 1), &[]);
            pass.set_bind_group(2, &atomics, &[]);
            pass.set_bind_group(3, &samplers, &[]);
            pass.draw(0..4, 0..1);
        }
        #[cfg(feature = "perf-diagnostics")]
        {
            backing.diagnostics.render_encode_ns = backing
                .diagnostics
                .render_encode_ns
                .saturating_add(elapsed_ns(render_encode_started));
            backing.diagnostics.total_ns = backing
                .diagnostics
                .total_ns
                .saturating_add(elapsed_ns(total_started));
        }
        AtomicPlaneReadbacks {
            coverage: coverage_readback,
            clip: clip_readback,
            color: color_readback,
        }
    }
}

const IMAGE_MESH_POSITION_ATTRIBUTE: [wgpu::VertexAttribute; 1] = [wgpu::VertexAttribute {
    format: wgpu::VertexFormat::Float32x2,
    offset: 0,
    shader_location: 0,
}];
const IMAGE_MESH_UV_ATTRIBUTE: [wgpu::VertexAttribute; 1] = [wgpu::VertexAttribute {
    format: wgpu::VertexFormat::Float32x2,
    offset: 0,
    shader_location: 1,
}];

fn image_mesh_vertex_layout(shader_location: u32) -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<[f32; 2]>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: match shader_location {
            0 => &IMAGE_MESH_POSITION_ATTRIBUTE,
            1 => &IMAGE_MESH_UV_ATTRIBUTE,
            _ => unreachable!("image mesh only has position and UV streams"),
        },
    }
}

fn shader(device: &wgpu::Device, label: &'static str, source: &'static str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

// C++ combines ENABLE_DITHER across an advanced atomic flush, so every draw
// pipeline in that flush must compile the dither branch.
const ADVANCED_FLUSH_DITHER: f64 = 1.0;

fn advanced_path_constants(feather: f64, hsl: f64) -> [(&'static str, f64); 7] {
    [
        ("0", 1.0),
        ("1", 1.0),
        ("2", 1.0),
        ("3", feather),
        ("4", 0.0),
        ("6", hsl),
        ("7", ADVANCED_FLUSH_DITHER),
    ]
}

fn advanced_fill_constants(hsl: f64) -> [(&'static str, f64); 6] {
    [
        ("0", 1.0),
        ("1", 1.0),
        ("2", 1.0),
        ("4", 0.0),
        ("6", hsl),
        ("7", ADVANCED_FLUSH_DITHER),
    ]
}

fn options<'a>(constants: &'a [(&'a str, f64)]) -> wgpu::PipelineCompilationOptions<'a> {
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
    device.create_counted_buffer_init(&wgpu::util::BufferInitDescriptor {
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
fn linear_sampler() -> wgpu::SamplerDescriptor<'static> {
    wgpu::SamplerDescriptor {
        label: Some("nuxie-atomic-linear-sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    }
}

#[cfg(feature = "perf-diagnostics")]
fn elapsed_ns(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

pub(crate) fn image_sampler(sampler: ImageSampler) -> wgpu::SamplerDescriptor<'static> {
    let address_mode = |wrap| match wrap {
        ImageWrap::Clamp => wgpu::AddressMode::ClampToEdge,
        ImageWrap::Repeat => wgpu::AddressMode::Repeat,
        ImageWrap::Mirror => wgpu::AddressMode::MirrorRepeat,
    };
    let filter = match sampler.filter {
        ImageFilter::Bilinear => wgpu::FilterMode::Linear,
        ImageFilter::Nearest => wgpu::FilterMode::Nearest,
    };
    wgpu::SamplerDescriptor {
        label: Some("nuxie-image-sampler"),
        address_mode_u: address_mode(sampler.wrap_x),
        address_mode_v: address_mode(sampler.wrap_y),
        mag_filter: filter,
        min_filter: filter,
        // Rive's Metal and WebGPU backends both use nearest mip selection;
        // ImageFilter only controls filtering within the selected level.
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    }
}

fn triangle_vertex_range(
    shared_flush_group: bool,
    shared_ranges: &[std::ops::Range<u32>],
    draw_index: usize,
    per_draw_vertex_count: u32,
) -> Option<std::ops::Range<u32>> {
    if shared_flush_group {
        let range = shared_ranges[draw_index].clone();
        (!range.is_empty()).then_some(range)
    } else {
        (per_draw_vertex_count != 0).then_some(0..per_draw_vertex_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advanced_atomic_draws_enable_flush_wide_dither() {
        for constants in [
            advanced_path_constants(0.0, 0.0).as_slice(),
            advanced_path_constants(1.0, 1.0).as_slice(),
            advanced_fill_constants(0.0).as_slice(),
            advanced_fill_constants(1.0).as_slice(),
        ] {
            assert_eq!(
                constants.iter().find(|(id, _)| *id == "7"),
                Some(&("7", 1.0))
            );
        }
    }

    #[test]
    fn bilinear_image_filter_keeps_nearest_mip_selection() {
        let descriptor = image_sampler(ImageSampler {
            filter: ImageFilter::Bilinear,
            wrap_x: ImageWrap::Clamp,
            wrap_y: ImageWrap::Clamp,
        });

        assert_eq!(descriptor.min_filter, wgpu::FilterMode::Linear);
        assert_eq!(descriptor.mag_filter, wgpu::FilterMode::Linear);
        assert_eq!(descriptor.mipmap_filter, wgpu::MipmapFilterMode::Nearest);
    }

    #[test]
    fn triangle_free_shared_batch_does_not_select_a_per_draw_buffer() {
        assert_eq!(triangle_vertex_range(true, &[0..0, 0..0], 1, 0), None);
        assert_eq!(triangle_vertex_range(true, &[0..3], 0, 3), Some(0..3));
        assert_eq!(triangle_vertex_range(false, &[], 0, 3), Some(0..3));
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
fn disabled_color_target() -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format: wgpu::TextureFormat::Rgba8Unorm,
        blend: None,
        write_mask: wgpu::ColorWrites::empty(),
    }
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
