//! Clockwise-atomic fill passes translated from Rive's Vulkan shader family.
//!
//! This is intentionally separate from `atomic_pipeline`: the two shader
//! families use incompatible coverage-buffer encodings and pass schedules.

use crate::gpu::{
    ContourData, FlushUniforms, PaintAuxData, PaintData, PatchVertex, PathData, TriangleVertex,
};
use wgpu::util::DeviceExt;

pub(crate) struct ClockwiseAtomicPipeline {
    borrowed_path: wgpu::RenderPipeline,
    borrowed_interior: wgpu::RenderPipeline,
    path: wgpu::RenderPipeline,
    interior: wgpu::RenderPipeline,
    clipped_path: wgpu::RenderPipeline,
    clipped_interior: wgpu::RenderPipeline,
    outer_clip_path: wgpu::RenderPipeline,
    outer_clip_interior: wgpu::RenderPipeline,
    nested_clip_path: wgpu::RenderPipeline,
    nested_clip_interior: wgpu::RenderPipeline,
    flush_layout: wgpu::BindGroupLayout,
    image_layout: wgpu::BindGroupLayout,
    clip_layout: wgpu::BindGroupLayout,
    sampled_clip_layout: wgpu::BindGroupLayout,
    sampler_layout: wgpu::BindGroupLayout,
}

pub(crate) struct ClockwiseAtomicDraw<'a> {
    pub tessellation: &'a wgpu::TextureView,
    pub borrowed_base_instance: u32,
    pub main_base_instance: u32,
    pub instance_count: u32,
    pub patch_index_range: std::ops::Range<u32>,
    pub borrowed_triangles: &'a [TriangleVertex],
    pub main_triangles: &'a [TriangleVertex],
    pub kind: ClockwiseAtomicDrawKind,
}

pub(crate) struct ClockwiseAtomicCoverageReadback {
    pub borrowed: wgpu::Buffer,
    pub main: wgpu::Buffer,
    pub word_count: usize,
    pub clip_updates: Vec<wgpu::Buffer>,
    pub clip_bytes_per_row: u32,
    pub clip_height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClockwiseAtomicDrawKind {
    Content,
    ClippedContent,
    OutermostClip,
    NestedClip,
}

impl ClockwiseAtomicPipeline {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let path_vertex = shader(
            device,
            "nuxie-cwa-path-vertex",
            include_str!("generated/clockwise_atomic_draw_path.webgpu_vert.wgsl"),
        );
        let path_fragment = shader(
            device,
            "nuxie-cwa-path-fragment",
            include_str!("generated/clockwise_atomic_draw_path.webgpu_fixedcolor_frag.wgsl"),
        );
        let borrowed_path_fragment = shader(
            device,
            "nuxie-cwa-borrowed-path-fragment",
            include_str!("generated/clockwise_atomic_draw_path_borrowed.webgpu_frag.wgsl"),
        );
        let interior_vertex = shader(
            device,
            "nuxie-cwa-interior-vertex",
            include_str!("generated/clockwise_atomic_draw_interior_triangles.webgpu_vert.wgsl"),
        );
        let interior_fragment = shader(
            device,
            "nuxie-cwa-interior-fragment",
            include_str!(
                "generated/clockwise_atomic_draw_interior_triangles.webgpu_fixedcolor_frag.wgsl"
            ),
        );
        let borrowed_interior_fragment = shader(
            device,
            "nuxie-cwa-borrowed-interior-fragment",
            include_str!(
                "generated/clockwise_atomic_draw_interior_triangles_borrowed.webgpu_frag.wgsl"
            ),
        );
        let sampled_clip_path_fragment = shader(
            device,
            "nuxie-cwa-sampled-clip-path-fragment",
            include_str!(
                "generated/clockwise_atomic_draw_path_sampled_clip.webgpu_fixedcolor_frag.wgsl"
            ),
        );
        let sampled_clip_interior_fragment = shader(
            device,
            "nuxie-cwa-sampled-clip-interior-fragment",
            include_str!(
                "generated/clockwise_atomic_draw_interior_triangles_sampled_clip.webgpu_fixedcolor_frag.wgsl"
            ),
        );
        let clip_path_fragment = shader(
            device,
            "nuxie-cwa-clip-path-fragment",
            include_str!("generated/clockwise_atomic_draw_clip.webgpu_fixedcolor_frag.wgsl"),
        );
        let clip_interior_fragment = shader(
            device,
            "nuxie-cwa-clip-interior-fragment",
            include_str!(
                "generated/clockwise_atomic_draw_clip_interior_triangles.webgpu_fixedcolor_frag.wgsl"
            ),
        );
        let flush_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-cwa-flush-layout"),
            entries: &[
                uniform_entry(0),
                storage_entry(3, true),
                storage_entry(4, true),
                storage_entry(5, true),
                storage_entry(6, true),
                storage_entry(7, false),
                texture_entry(8, wgpu::TextureSampleType::Uint),
                texture_entry(9, wgpu::TextureSampleType::Float { filterable: true }),
                texture_entry(10, wgpu::TextureSampleType::Float { filterable: true }),
                texture_entry(11, wgpu::TextureSampleType::Float { filterable: true }),
            ],
        });
        let image_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-cwa-image-layout"),
            entries: &[
                texture_entry(12, wgpu::TextureSampleType::Float { filterable: true }),
                sampler_entry(14),
            ],
        });
        let clip_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-cwa-clip-layout"),
            entries: &[storage_entry(1, false)],
        });
        let sampled_clip_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("nuxie-cwa-sampled-clip-layout"),
                entries: &[texture_entry(
                    1,
                    wgpu::TextureSampleType::Float { filterable: false },
                )],
            });
        let sampler_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-cwa-sampler-layout"),
            entries: &[sampler_entry(9), sampler_entry(10), sampler_entry(11)],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nuxie-cwa-pipeline-layout"),
            bind_group_layouts: &[
                Some(&flush_layout),
                Some(&image_layout),
                Some(&clip_layout),
                Some(&sampler_layout),
            ],
            immediate_size: 0,
        });
        let sampled_clip_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("nuxie-cwa-sampled-clip-pipeline-layout"),
                bind_group_layouts: &[
                    Some(&flush_layout),
                    Some(&image_layout),
                    Some(&sampled_clip_layout),
                    Some(&sampler_layout),
                ],
                immediate_size: 0,
            });
        let path_options = options(&[("0", 0.0), ("1", 0.0), ("3", 0.0), ("7", 0.0)]);
        let interior_options = options(&[("0", 0.0), ("1", 0.0), ("7", 0.0)]);
        let path = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-cwa-path-pipeline"),
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
                compilation_options: path_options,
                targets: &[Some(color_target(wgpu::ColorWrites::ALL))],
            }),
            multiview_mask: None,
            cache: None,
        });
        let borrowed_path = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-cwa-borrowed-path-pipeline"),
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
                module: &borrowed_path_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[("3", 0.0)]),
                targets: &[Some(color_target(wgpu::ColorWrites::empty()))],
            }),
            multiview_mask: None,
            cache: None,
        });
        let interior = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-cwa-interior-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &interior_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(TriangleVertex::layout())],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &interior_fragment,
                entry_point: Some("main"),
                compilation_options: interior_options,
                targets: &[Some(color_target(wgpu::ColorWrites::ALL))],
            }),
            multiview_mask: None,
            cache: None,
        });
        let borrowed_interior = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-cwa-borrowed-interior-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &interior_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(TriangleVertex::layout())],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &borrowed_interior_fragment,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[Some(color_target(wgpu::ColorWrites::empty()))],
            }),
            multiview_mask: None,
            cache: None,
        });
        let clipped_path = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-cwa-clipped-path-pipeline"),
            layout: Some(&sampled_clip_pipeline_layout),
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
                module: &sampled_clip_path_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[("0", 1.0), ("1", 0.0), ("3", 0.0), ("7", 0.0)]),
                targets: &[Some(color_target(wgpu::ColorWrites::ALL))],
            }),
            multiview_mask: None,
            cache: None,
        });
        let clipped_interior = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-cwa-clipped-interior-pipeline"),
            layout: Some(&sampled_clip_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &interior_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(TriangleVertex::layout())],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &sampled_clip_interior_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[("0", 1.0), ("1", 0.0), ("7", 0.0)]),
                targets: &[Some(color_target(wgpu::ColorWrites::ALL))],
            }),
            multiview_mask: None,
            cache: None,
        });
        let make_clip_path = |label, nested| {
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
                    cull_mode: Some(wgpu::Face::Front),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &clip_path_fragment,
                    entry_point: Some("main"),
                    compilation_options: options(&[("9", if nested { 1.0 } else { 0.0 })]),
                    targets: &[
                        Some(color_target(wgpu::ColorWrites::empty())),
                        Some(if nested {
                            clip_min_target()
                        } else {
                            clip_plus_target()
                        }),
                    ],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let make_clip_interior = |label, nested| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &interior_vertex,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(TriangleVertex::layout())],
                },
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Front),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &clip_interior_fragment,
                    entry_point: Some("main"),
                    compilation_options: options(&[("9", if nested { 1.0 } else { 0.0 })]),
                    targets: &[
                        Some(color_target(wgpu::ColorWrites::empty())),
                        Some(if nested {
                            clip_min_target()
                        } else {
                            clip_plus_target()
                        }),
                    ],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let outer_clip_path = make_clip_path("nuxie-cwa-outer-clip-path-pipeline", false);
        let nested_clip_path = make_clip_path("nuxie-cwa-nested-clip-path-pipeline", true);
        let outer_clip_interior =
            make_clip_interior("nuxie-cwa-outer-clip-interior-pipeline", false);
        let nested_clip_interior =
            make_clip_interior("nuxie-cwa-nested-clip-interior-pipeline", true);
        Self {
            borrowed_path,
            borrowed_interior,
            path,
            interior,
            clipped_path,
            clipped_interior,
            outer_clip_path,
            outer_clip_interior,
            nested_clip_path,
            nested_clip_interior,
            flush_layout,
            image_layout,
            clip_layout,
            sampled_clip_layout,
            sampler_layout,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_fills(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        feather_lut: &wgpu::TextureView,
        patch_vertices: &wgpu::Buffer,
        patch_indices: &wgpu::Buffer,
        draws: &[ClockwiseAtomicDraw<'_>],
        uniforms: &FlushUniforms,
        paths: &[PathData],
        paints: &[PaintData],
        paint_aux: &[PaintAuxData],
        contours: &[ContourData],
        coverage_word_count: usize,
        capture_coverage: bool,
    ) -> Option<ClockwiseAtomicCoverageReadback> {
        assert!(!draws.is_empty());
        assert_ne!(uniforms.coverage_buffer_prefix, 0);
        let uniform = upload(
            device,
            "nuxie-cwa-uniforms",
            std::slice::from_ref(uniforms),
            wgpu::BufferUsages::UNIFORM,
        );
        let paths = upload(
            device,
            "nuxie-cwa-paths",
            paths,
            wgpu::BufferUsages::STORAGE,
        );
        let paints = upload(
            device,
            "nuxie-cwa-paints",
            paints,
            wgpu::BufferUsages::STORAGE,
        );
        let paint_aux = upload(
            device,
            "nuxie-cwa-paint-aux",
            paint_aux,
            wgpu::BufferUsages::STORAGE,
        );
        let contours = upload(
            device,
            "nuxie-cwa-contours",
            contours,
            wgpu::BufferUsages::STORAGE,
        );
        let coverage = upload(
            device,
            "nuxie-cwa-coverage",
            &vec![0u32; coverage_word_count.max(1)],
            wgpu::BufferUsages::STORAGE
                | if capture_coverage {
                    wgpu::BufferUsages::COPY_SRC
                } else {
                    wgpu::BufferUsages::empty()
                },
        );
        let coverage_readback = capture_coverage.then(|| {
            let size = (coverage_word_count.max(1) * std::mem::size_of::<u32>()) as u64;
            let make_buffer = |label| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label),
                    size,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                })
            };
            ClockwiseAtomicCoverageReadback {
                borrowed: make_buffer("nuxie-cwa-borrowed-coverage-readback"),
                main: make_buffer("nuxie-cwa-main-coverage-readback"),
                word_count: coverage_word_count.max(1),
                clip_updates: Vec::new(),
                clip_bytes_per_row: 0,
                clip_height: 0,
            }
        });
        // Unclipped fixed-color shaders retain the upstream no-op clip-plane
        // store. Keep that family on a scratch plane; clipped shaders bind the
        // render-attachment texture below through a separate layout.
        let scratch_clip = upload(
            device,
            "nuxie-cwa-scratch-clip",
            &vec![0u32; (uniforms.render_target_width * uniforms.render_target_height) as usize],
            wgpu::BufferUsages::STORAGE,
        );
        let clip_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-cwa-clip-texture"),
            size: wgpu::Extent3d {
                width: uniforms.render_target_width,
                height: uniforms.render_target_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | if capture_coverage {
                    wgpu::TextureUsages::COPY_SRC
                } else {
                    wgpu::TextureUsages::empty()
                },
            view_formats: &[],
        });
        let clip_view = clip_texture.create_view(&Default::default());
        let dummy = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-cwa-dummy-texture"),
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
        let sampler = device.create_sampler(&linear_sampler());
        let flush_groups = draws
            .iter()
            .map(|draw| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("nuxie-cwa-flush-group"),
                    layout: &self.flush_layout,
                    entries: &[
                        binding(0, uniform.as_entire_binding()),
                        binding(3, paths.as_entire_binding()),
                        binding(4, paints.as_entire_binding()),
                        binding(5, paint_aux.as_entire_binding()),
                        binding(6, contours.as_entire_binding()),
                        binding(7, coverage.as_entire_binding()),
                        binding(8, wgpu::BindingResource::TextureView(draw.tessellation)),
                        binding(9, wgpu::BindingResource::TextureView(&dummy_view)),
                        binding(10, wgpu::BindingResource::TextureView(feather_lut)),
                        binding(11, wgpu::BindingResource::TextureView(&dummy_view)),
                    ],
                })
            })
            .collect::<Vec<_>>();
        let image = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-cwa-image-group"),
            layout: &self.image_layout,
            entries: &[
                binding(12, wgpu::BindingResource::TextureView(&dummy_view)),
                binding(14, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        let clip = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-cwa-clip-group"),
            layout: &self.clip_layout,
            entries: &[binding(1, scratch_clip.as_entire_binding())],
        });
        let sampled_clip = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-cwa-sampled-clip-group"),
            layout: &self.sampled_clip_layout,
            entries: &[binding(1, wgpu::BindingResource::TextureView(&clip_view))],
        });
        let samplers = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-cwa-sampler-group"),
            layout: &self.sampler_layout,
            entries: &[
                binding(9, wgpu::BindingResource::Sampler(&sampler)),
                binding(10, wgpu::BindingResource::Sampler(&sampler)),
                binding(11, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        let borrowed_triangle_buffers = draws
            .iter()
            .map(|draw| upload_optional_triangles(device, draw.borrowed_triangles))
            .collect::<Vec<_>>();
        let main_triangle_buffers = draws
            .iter()
            .map(|draw| upload_optional_triangles(device, draw.main_triangles))
            .collect::<Vec<_>>();
        let clip_bytes_per_row = (uniforms.render_target_width * 4)
            .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let mut clip_update_readbacks = Vec::new();

        for (index, draw) in draws.iter().enumerate() {
            if draw.kind == ClockwiseAtomicDrawKind::OutermostClip {
                continue;
            }
            let attachments = [color_attachment(target)];
            let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                "nuxie-cwa-borrowed-path-pass",
                &attachments,
            ));
            pass.set_pipeline(&self.borrowed_path);
            set_groups(&mut pass, &flush_groups[index], &image, &clip, &samplers);
            pass.set_vertex_buffer(0, patch_vertices.slice(..));
            pass.set_index_buffer(patch_indices.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(
                draw.patch_index_range.clone(),
                0,
                draw.borrowed_base_instance..draw.borrowed_base_instance + draw.instance_count,
            );
            drop(pass);
            if let Some(buffer) = &borrowed_triangle_buffers[index] {
                let attachments = [color_attachment(target)];
                let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                    "nuxie-cwa-borrowed-interior-pass",
                    &attachments,
                ));
                pass.set_pipeline(&self.borrowed_interior);
                set_groups(&mut pass, &flush_groups[index], &image, &clip, &samplers);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..draw.borrowed_triangles.len() as u32, 0..1);
            }
        }

        if let Some(readback) = &coverage_readback {
            encoder.copy_buffer_to_buffer(&coverage, 0, &readback.borrowed, 0, coverage.size());
        }

        for (index, draw) in draws.iter().enumerate() {
            match draw.kind {
                ClockwiseAtomicDrawKind::Content | ClockwiseAtomicDrawKind::ClippedContent => {
                    let attachments = [color_attachment(target)];
                    let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                        "nuxie-cwa-main-path-pass",
                        &attachments,
                    ));
                    pass.set_pipeline(if draw.kind == ClockwiseAtomicDrawKind::ClippedContent {
                        &self.clipped_path
                    } else {
                        &self.path
                    });
                    if draw.kind == ClockwiseAtomicDrawKind::ClippedContent {
                        set_groups(
                            &mut pass,
                            &flush_groups[index],
                            &image,
                            &sampled_clip,
                            &samplers,
                        );
                    } else {
                        set_groups(&mut pass, &flush_groups[index], &image, &clip, &samplers);
                    }
                    pass.set_vertex_buffer(0, patch_vertices.slice(..));
                    pass.set_index_buffer(patch_indices.slice(..), wgpu::IndexFormat::Uint16);
                    pass.draw_indexed(
                        draw.patch_index_range.clone(),
                        0,
                        draw.main_base_instance..draw.main_base_instance + draw.instance_count,
                    );
                    drop(pass);
                    if let Some(buffer) = &main_triangle_buffers[index] {
                        let attachments = [color_attachment(target)];
                        let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                            "nuxie-cwa-main-interior-pass",
                            &attachments,
                        ));
                        pass.set_pipeline(
                            if draw.kind == ClockwiseAtomicDrawKind::ClippedContent {
                                &self.clipped_interior
                            } else {
                                &self.interior
                            },
                        );
                        if draw.kind == ClockwiseAtomicDrawKind::ClippedContent {
                            set_groups(
                                &mut pass,
                                &flush_groups[index],
                                &image,
                                &sampled_clip,
                                &samplers,
                            );
                        } else {
                            set_groups(&mut pass, &flush_groups[index], &image, &clip, &samplers);
                        }
                        pass.set_vertex_buffer(0, buffer.slice(..));
                        pass.draw(0..draw.main_triangles.len() as u32, 0..1);
                    }
                }
                ClockwiseAtomicDrawKind::OutermostClip | ClockwiseAtomicDrawKind::NestedClip => {
                    let nested = draw.kind == ClockwiseAtomicDrawKind::NestedClip;
                    let clip_load = if nested {
                        wgpu::LoadOp::Load
                    } else {
                        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                    };
                    let attachments = [
                        color_attachment(target),
                        color_attachment_with_load(&clip_view, clip_load),
                    ];
                    let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                        "nuxie-cwa-clip-path-pass",
                        &attachments,
                    ));
                    pass.set_pipeline(if nested {
                        &self.nested_clip_path
                    } else {
                        &self.outer_clip_path
                    });
                    set_groups(&mut pass, &flush_groups[index], &image, &clip, &samplers);
                    pass.set_vertex_buffer(0, patch_vertices.slice(..));
                    pass.set_index_buffer(patch_indices.slice(..), wgpu::IndexFormat::Uint16);
                    pass.draw_indexed(
                        draw.patch_index_range.clone(),
                        0,
                        draw.main_base_instance..draw.main_base_instance + draw.instance_count,
                    );
                    drop(pass);
                    if let Some(buffer) = &main_triangle_buffers[index] {
                        let attachments = [
                            color_attachment(target),
                            color_attachment_with_load(&clip_view, wgpu::LoadOp::Load),
                        ];
                        let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                            "nuxie-cwa-clip-interior-pass",
                            &attachments,
                        ));
                        pass.set_pipeline(if nested {
                            &self.nested_clip_interior
                        } else {
                            &self.outer_clip_interior
                        });
                        set_groups(&mut pass, &flush_groups[index], &image, &clip, &samplers);
                        pass.set_vertex_buffer(0, buffer.slice(..));
                        pass.draw(0..draw.main_triangles.len() as u32, 0..1);
                    }
                    if capture_coverage && nested {
                        let readback = device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("nuxie-cwa-nested-clip-readback"),
                            size: u64::from(clip_bytes_per_row)
                                * u64::from(uniforms.render_target_height),
                            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                            mapped_at_creation: false,
                        });
                        encoder.copy_texture_to_buffer(
                            clip_texture.as_image_copy(),
                            wgpu::TexelCopyBufferInfo {
                                buffer: &readback,
                                layout: wgpu::TexelCopyBufferLayout {
                                    offset: 0,
                                    bytes_per_row: Some(clip_bytes_per_row),
                                    rows_per_image: Some(uniforms.render_target_height),
                                },
                            },
                            clip_texture.size(),
                        );
                        clip_update_readbacks.push(readback);
                    }
                }
            }
        }
        if let Some(readback) = &coverage_readback {
            encoder.copy_buffer_to_buffer(&coverage, 0, &readback.main, 0, coverage.size());
        }
        coverage_readback.map(|mut readback| {
            readback.clip_updates = clip_update_readbacks;
            readback.clip_bytes_per_row = clip_bytes_per_row;
            readback.clip_height = uniforms.render_target_height;
            readback
        })
    }
}

fn upload_optional_triangles(
    device: &wgpu::Device,
    values: &[TriangleVertex],
) -> Option<wgpu::Buffer> {
    (!values.is_empty()).then(|| {
        upload(
            device,
            "nuxie-cwa-triangles",
            values,
            wgpu::BufferUsages::VERTEX,
        )
    })
}

fn set_groups<'a>(
    pass: &mut wgpu::RenderPass<'a>,
    flush: &'a wgpu::BindGroup,
    image: &'a wgpu::BindGroup,
    clip: &'a wgpu::BindGroup,
    samplers: &'a wgpu::BindGroup,
) {
    pass.set_bind_group(0, flush, &[]);
    pass.set_bind_group(1, image, &[]);
    pass.set_bind_group(2, clip, &[]);
    pass.set_bind_group(3, samplers, &[]);
}

fn shader(device: &wgpu::Device, label: &'static str, source: &'static str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
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

fn color_target(write_mask: wgpu::ColorWrites) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format: wgpu::TextureFormat::Rgba8Unorm,
        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        write_mask,
    }
}

fn clip_plus_target() -> wgpu::ColorTargetState {
    let add = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    };
    wgpu::ColorTargetState {
        format: wgpu::TextureFormat::Rgba8Unorm,
        blend: Some(wgpu::BlendState {
            color: add,
            alpha: add,
        }),
        write_mask: wgpu::ColorWrites::ALL,
    }
}

fn clip_min_target() -> wgpu::ColorTargetState {
    let min = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Min,
    };
    wgpu::ColorTargetState {
        format: wgpu::TextureFormat::Rgba8Unorm,
        blend: Some(wgpu::BlendState {
            color: min,
            alpha: min,
        }),
        write_mask: wgpu::ColorWrites::ALL,
    }
}

fn linear_sampler() -> wgpu::SamplerDescriptor<'static> {
    wgpu::SamplerDescriptor {
        label: Some("nuxie-cwa-linear-sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    }
}

fn color_attachment(view: &wgpu::TextureView) -> Option<wgpu::RenderPassColorAttachment<'_>> {
    color_attachment_with_load(view, wgpu::LoadOp::Load)
}

fn color_attachment_with_load(
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
