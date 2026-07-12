//! Clockwise-atomic draw and resolve translated from Rive's WebGPU shaders.

use crate::gpu::{
    ContourData, FlushUniforms, ImageDrawUniforms, ImageRectVertex, PaintAuxData, PaintData,
    PatchVertex, PathData, TriangleVertex,
};
use bytemuck::Zeroable;
use nuxie_render_api::{ImageFilter, ImageSampler, ImageWrap};
use wgpu::util::DeviceExt;

pub(crate) struct AtomicPipeline {
    path: wgpu::RenderPipeline,
    outer_path: wgpu::RenderPipeline,
    feather_path: wgpu::RenderPipeline,
    feather_stroke_path: wgpu::RenderPipeline,
    stroke_path: wgpu::RenderPipeline,
    interior: wgpu::RenderPipeline,
    atlas_blit: wgpu::RenderPipeline,
    image_rect: wgpu::RenderPipeline,
    image_mesh: wgpu::RenderPipeline,
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
    pub image: Option<&'a wgpu::TextureView>,
    pub image_sampler: ImageSampler,
    pub image_uniforms: Option<ImageDrawUniforms>,
    pub image_mesh: Option<ImageMeshBuffers<'a>>,
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
            entries: &[storage_entry(1, false), storage_entry(3, false)],
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
        let dummy_image_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                cull_mode: Some(wgpu::Face::Back),
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
                cull_mode: Some(wgpu::Face::Back),
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
            primitive: wgpu::PrimitiveState::default(),
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
        Self {
            path,
            outer_path,
            feather_path,
            feather_stroke_path,
            stroke_path,
            interior,
            atlas_blit,
            image_rect,
            image_mesh,
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
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_batch(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        feather_lut: &wgpu::TextureView,
        patch_vertices: &wgpu::Buffer,
        patch_indices: &wgpu::Buffer,
        draws: &[AtomicDraw<'_>],
        uniforms: &FlushUniforms,
        paths: &[PathData],
        paints: &[PaintData],
        paint_aux: &[PaintAuxData],
        contours: &[ContourData],
        pixel_count: usize,
    ) {
        assert!(!draws.is_empty());
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
        let dummy_contours = [ContourData::zeroed()];
        let contours = upload(
            device,
            "nuxie-atomic-contours",
            if contours.is_empty() {
                &dummy_contours
            } else {
                contours
            },
            wgpu::BufferUsages::STORAGE,
        );
        let clips = upload(
            device,
            "nuxie-atomic-clips",
            &vec![0u32; pixel_count],
            wgpu::BufferUsages::STORAGE,
        );
        let coverage = upload(
            device,
            "nuxie-atomic-coverage",
            &vec![0u32; pixel_count],
            wgpu::BufferUsages::STORAGE,
        );
        let triangle_buffers = draws
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
        let sampler = device.create_sampler(&linear_sampler());
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
        let shared_flush_group = draws.iter().all(|draw| {
            std::ptr::eq(draw.tessellation, draws[0].tessellation)
                && draw.atlas.is_none()
                && draws[0].atlas.is_none()
                && draw.image.is_none()
        });
        let make_flush_group = |draw_index: usize, draw: &AtomicDraw<'_>| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("nuxie-atomic-flush-group"),
                layout: &self.flush_layout,
                entries: &[
                    binding(0, uniform.as_entire_binding()),
                    binding(
                        2,
                        image_uniform_buffers[draw_index]
                            .as_ref()
                            .unwrap_or(&self.dummy_image_uniforms)
                            .as_entire_binding(),
                    ),
                    binding(3, paths.as_entire_binding()),
                    binding(4, paints.as_entire_binding()),
                    binding(5, paint_aux.as_entire_binding()),
                    binding(6, contours.as_entire_binding()),
                    binding(8, wgpu::BindingResource::TextureView(draw.tessellation)),
                    binding(9, wgpu::BindingResource::TextureView(&dummy_view)),
                    binding(10, wgpu::BindingResource::TextureView(feather_lut)),
                    binding(
                        11,
                        wgpu::BindingResource::TextureView(draw.atlas.unwrap_or(&dummy_view)),
                    ),
                ],
            })
        };
        let flush_groups = if shared_flush_group {
            vec![make_flush_group(0, &draws[0])]
        } else {
            draws
                .iter()
                .enumerate()
                .map(|(index, draw)| make_flush_group(index, draw))
                .collect::<Vec<_>>()
        };
        let flush_group_index = |draw_index: usize| if shared_flush_group { 0 } else { draw_index };
        let image_groups = draws
            .iter()
            .map(|draw| {
                draw.image.map(|image| {
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        let image_group = |draw_index: usize| {
            image_groups[draw_index]
                .as_ref()
                .unwrap_or(&self.dummy_image_group)
        };
        let atomics = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-atomic-buffer-group"),
            layout: &self.atomic_layout,
            entries: &[
                binding(1, clips.as_entire_binding()),
                binding(3, coverage.as_entire_binding()),
            ],
        });
        let samplers = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-atomic-sampler-group"),
            layout: &self.sampler_layout,
            entries: &[
                binding(9, wgpu::BindingResource::Sampler(&sampler)),
                binding(10, wgpu::BindingResource::Sampler(&sampler)),
                binding(11, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        for (draw_index, draw) in draws.iter().enumerate() {
            if draw.atlas.is_none() {
                continue;
            }
            let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
            let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                "nuxie-atomic-atlas-blit-pass",
                &attachments,
            ));
            pass.set_pipeline(&self.atlas_blit);
            pass.set_bind_group(0, &flush_groups[flush_group_index(draw_index)], &[]);
            pass.set_bind_group(1, image_group(draw_index), &[]);
            pass.set_bind_group(2, &atomics, &[]);
            pass.set_bind_group(3, &samplers, &[]);
            pass.set_vertex_buffer(0, triangle_buffers[draw_index].as_ref().unwrap().slice(..));
            pass.draw(0..draw.atlas_blit_vertices.len() as u32, 0..1);
        }
        if shared_flush_group && draws.iter().any(|draw| draw.atlas.is_none()) {
            let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
            let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                "nuxie-atomic-path-pass",
                &attachments,
            ));
            pass.set_bind_group(1, image_group(0), &[]);
            pass.set_bind_group(2, &atomics, &[]);
            pass.set_bind_group(3, &samplers, &[]);
            pass.set_vertex_buffer(0, patch_vertices.slice(..));
            pass.set_index_buffer(patch_indices.slice(..), wgpu::IndexFormat::Uint16);
            for (draw_index, draw) in draws.iter().enumerate() {
                if draw.atlas.is_some() {
                    continue;
                }
                pass.set_pipeline(if draw.is_feather && draw.is_stroke {
                    &self.feather_stroke_path
                } else if draw.is_feather {
                    &self.feather_path
                } else if draw.is_stroke {
                    &self.stroke_path
                } else if !draw.triangle_vertices.is_empty() {
                    &self.outer_path
                } else {
                    &self.path
                });
                pass.set_bind_group(0, &flush_groups[flush_group_index(draw_index)], &[]);
                pass.set_vertex_buffer(0, patch_vertices.slice(..));
                pass.set_index_buffer(patch_indices.slice(..), wgpu::IndexFormat::Uint16);
                pass.draw_indexed(
                    draw.patch_index_range.clone(),
                    0,
                    draw.base_instance..draw.base_instance + draw.instance_count,
                );
                if let Some(triangle_buffer) = &triangle_buffers[draw_index] {
                    pass.set_pipeline(&self.interior);
                    pass.set_vertex_buffer(0, triangle_buffer.slice(..));
                    pass.draw(0..draw.triangle_vertices.len() as u32, 0..1);
                }
            }
        } else {
            for (draw_index, draw) in draws.iter().enumerate() {
                if draw.atlas.is_some() {
                    continue;
                }
                if let Some(mesh) = draw.image_mesh {
                    let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
                    let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                        "nuxie-atomic-image-mesh-pass",
                        &attachments,
                    ));
                    pass.set_pipeline(&self.image_mesh);
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
                    let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                        "nuxie-atomic-image-rect-pass",
                        &attachments,
                    ));
                    pass.set_pipeline(&self.image_rect);
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
                let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                    "nuxie-atomic-path-pass",
                    &attachments,
                ));
                pass.set_pipeline(if draw.is_feather && draw.is_stroke {
                    &self.feather_stroke_path
                } else if draw.is_feather {
                    &self.feather_path
                } else if draw.is_stroke {
                    &self.stroke_path
                } else if !draw.triangle_vertices.is_empty() {
                    &self.outer_path
                } else {
                    &self.path
                });
                pass.set_bind_group(0, &flush_groups[flush_group_index(draw_index)], &[]);
                pass.set_bind_group(1, image_group(draw_index), &[]);
                pass.set_bind_group(2, &atomics, &[]);
                pass.set_bind_group(3, &samplers, &[]);
                pass.set_vertex_buffer(0, patch_vertices.slice(..));
                pass.set_index_buffer(patch_indices.slice(..), wgpu::IndexFormat::Uint16);
                pass.draw_indexed(
                    draw.patch_index_range.clone(),
                    0,
                    draw.base_instance..draw.base_instance + draw.instance_count,
                );
                drop(pass);
                if let Some(triangle_buffer) = &triangle_buffers[draw_index] {
                    let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
                    let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                        "nuxie-atomic-interior-pass",
                        &attachments,
                    ));
                    pass.set_pipeline(&self.interior);
                    pass.set_bind_group(0, &flush_groups[flush_group_index(draw_index)], &[]);
                    pass.set_bind_group(1, image_group(draw_index), &[]);
                    pass.set_bind_group(2, &atomics, &[]);
                    pass.set_bind_group(3, &samplers, &[]);
                    pass.set_vertex_buffer(0, triangle_buffer.slice(..));
                    pass.draw(0..draw.triangle_vertices.len() as u32, 0..1);
                }
            }
        }
        {
            let attachments = [color_attachment(target, wgpu::LoadOp::Load)];
            let mut pass = encoder.begin_render_pass(&render_pass_descriptor(
                "nuxie-atomic-resolve-pass",
                &attachments,
            ));
            pass.set_pipeline(if draws.iter().any(|draw| draw.is_feather) {
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
fn linear_sampler() -> wgpu::SamplerDescriptor<'static> {
    wgpu::SamplerDescriptor {
        label: Some("nuxie-atomic-linear-sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    }
}

fn image_sampler(sampler: ImageSampler) -> wgpu::SamplerDescriptor<'static> {
    let address_mode = |wrap| match wrap {
        ImageWrap::Clamp => wgpu::AddressMode::ClampToEdge,
        ImageWrap::Repeat => wgpu::AddressMode::Repeat,
        ImageWrap::Mirror => wgpu::AddressMode::MirrorRepeat,
    };
    let filter = match sampler.filter {
        ImageFilter::Bilinear => wgpu::FilterMode::Linear,
        ImageFilter::Nearest => wgpu::FilterMode::Nearest,
    };
    let mipmap_filter = match sampler.filter {
        ImageFilter::Bilinear => wgpu::MipmapFilterMode::Linear,
        ImageFilter::Nearest => wgpu::MipmapFilterMode::Nearest,
    };
    wgpu::SamplerDescriptor {
        label: Some("nuxie-image-sampler"),
        address_mode_u: address_mode(sampler.wrap_x),
        address_mode_v: address_mode(sampler.wrap_y),
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter,
        ..Default::default()
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
