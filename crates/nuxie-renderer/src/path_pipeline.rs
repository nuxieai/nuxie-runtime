//! MSAA analytic path draw translated from `draw_path.vert` and `draw_msaa_object.frag`.

use crate::{
    atomic_pipeline::image_sampler,
    gpu::{ContourData, FlushUniforms, PaintAuxData, PaintData, PatchVertex, PathData},
    tessellator::TessellationUploadFrame,
    work_metrics::{CountedDeviceExt, CountedRenderPass},
};
use nuxie_render_api::{ImageFilter, ImageSampler, ImageWrap};

// C++ includes ENABLE_DITHER in fixed-color MSAA path batches.
const FIXED_COLOR_FRAGMENT_CONSTANTS: [(&str, f64); 2] = [("2", 0.0), ("7", 1.0)];

pub(crate) struct PathPipeline {
    stroke: DirectPipelineSet,
    fill_borrowed: DirectPipelineSet,
    fill_forward: DirectPipelineSet,
    fill_cleanup: DirectPipelineSet,
    clockwise_fill_cleanup: DirectPipelineSet,
    even_odd_fill_stencil: DirectPipelineSet,
    even_odd_fill_cover: DirectPipelineSet,
    pub clip_borrowed_pipeline: wgpu::RenderPipeline,
    pub clip_update_pipeline: wgpu::RenderPipeline,
    pub clip_cleanup_pipeline: wgpu::RenderPipeline,
    pub clockwise_clip_cleanup_pipeline: wgpu::RenderPipeline,
    pub even_odd_clip_stencil_pipeline: wgpu::RenderPipeline,
    pub even_odd_clip_cover_pipeline: wgpu::RenderPipeline,
    pub nested_clip_pipeline: wgpu::RenderPipeline,
    pub nested_even_odd_clip_pipeline: wgpu::RenderPipeline,
    flush_layout: wgpu::BindGroupLayout,
    image_layout: wgpu::BindGroupLayout,
    // C++ owns its null texture and sampler bindings on the WebGPU context.
    _dummy_texture: wgpu::Texture,
    dummy_view: wgpu::TextureView,
    dummy_image_group: wgpu::BindGroup,
    image_samplers: Vec<wgpu::Sampler>,
    sampler_group: wgpu::BindGroup,
}

struct DirectPipelineSet {
    fixed: PipelineVariants,
    opaque_fixed: PipelineVariants,
    advanced: Option<PipelineVariants>,
    advanced_hsl: Option<PipelineVariants>,
}

struct PipelineVariants {
    unclipped: wgpu::RenderPipeline,
    unclipped_rect: Option<wgpu::RenderPipeline>,
    path_clip: wgpu::RenderPipeline,
    path_clip_rect: Option<wgpu::RenderPipeline>,
}

#[derive(Clone, Copy)]
pub(crate) enum DirectPathPipelineKind {
    Stroke,
    FillBorrowed,
    FillForward,
    FillCleanup,
    ClockwiseFillCleanup,
    EvenOddFillStencil,
    EvenOddFillCover,
}

pub(crate) struct PreparedPathDraw {
    pub flush_group: wgpu::BindGroup,
    pub image_group: wgpu::BindGroup,
    pub sampler_group: wgpu::BindGroup,
    pub base_instance: u32,
    pub instance_count: u32,
    has_image: bool,
}

impl PreparedPathDraw {
    pub(crate) fn bind_resources(&self, pass: &mut CountedRenderPass<'_>, bind_all: bool) {
        if bind_all {
            pass.set_bind_group(0, &self.flush_group, &[]);
        }
        if bind_all || self.has_image {
            pass.set_bind_group(1, &self.image_group, &[]);
        }
        if bind_all {
            pass.set_bind_group(3, &self.sampler_group, &[]);
        }
    }
}

pub(crate) struct PreparedPathResources {
    flush_group: wgpu::BindGroup,
}

impl PathPipeline {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let no_clip_vertex = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nuxie-msaa-path-vertex"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("generated/draw_msaa_path.webgpu_noclipdistance_vert.wgsl").into(),
            ),
        });
        let clip_rect_vertex = device
            .features()
            .contains(wgpu::Features::CLIP_DISTANCES)
            .then(|| {
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("nuxie-msaa-path-clip-rect-vertex"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("generated/draw_msaa_path.webgpu_vert.wgsl").into(),
                    ),
                })
            });
        let fixed_fragment = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nuxie-msaa-path-fragment"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("generated/draw_msaa_path.webgpu_fixedcolor_frag.wgsl").into(),
            ),
        });
        let advanced_fragment = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nuxie-msaa-path-advanced-fragment"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("generated/draw_msaa_path.webgpu_frag.wgsl").into(),
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
                texture_entry(13, wgpu::TextureSampleType::Float { filterable: false }),
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
        let keep = wgpu::StencilOperation::Keep;
        let stencil_face = |compare, fail_op, pass_op| wgpu::StencilFaceState {
            compare,
            fail_op,
            depth_fail_op: keep,
            pass_op,
        };
        let disabled_face = stencil_face(wgpu::CompareFunction::Always, keep, keep);
        let stencil_state = |front, back, read_mask, write_mask| wgpu::StencilState {
            front,
            back,
            read_mask,
            write_mask,
        };
        let create_pipeline = |label: &'static str,
                               vertex: &wgpu::ShaderModule,
                               fragment: &wgpu::ShaderModule,
                               clip_rect,
                               advanced_blend,
                               hsl_blend,
                               cull_mode,
                               stencil,
                               depth_compare,
                               depth_write_enabled,
                               color_write_mask,
                               blend_enabled| {
            let vertex_constants: &[(&str, f64)] = match (clip_rect, advanced_blend) {
                (true, true) => &[("0", 0.0), ("1", 1.0), ("2", 1.0)],
                (true, false) => &[("0", 0.0), ("1", 1.0), ("2", 0.0)],
                (false, true) => &[("0", 0.0), ("2", 1.0)],
                (false, false) => &[("0", 0.0), ("2", 0.0)],
            };
            let advanced_fragment_constants = [
                ("2", 1.0),
                ("6", if hsl_blend { 1.0 } else { 0.0 }),
                ("7", 1.0),
            ];
            let fragment_constants = if advanced_blend {
                &advanced_fragment_constants[..]
            } else {
                &FIXED_COLOR_FRAGMENT_CONSTANTS[..]
            };
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: vertex,
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: vertex_constants,
                        ..Default::default()
                    },
                    buffers: &[Some(PatchVertex::layout())],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: Some(depth_write_enabled),
                    depth_compare: Some(depth_compare),
                    stencil,
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 4,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: fragment,
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: fragment_constants,
                        ..Default::default()
                    },
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: (blend_enabled && color_write_mask == wgpu::ColorWrites::ALL)
                            .then_some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: color_write_mask,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let create_direct_variants = |label: &'static str,
                                      fragment: &wgpu::ShaderModule,
                                      advanced_blend,
                                      hsl_blend,
                                      cull_mode,
                                      stencil: wgpu::StencilState,
                                      path_clip_stencil: wgpu::StencilState,
                                      depth_compare,
                                      depth_write_enabled,
                                      color_write_mask,
                                      blend_enabled| {
            let unclipped = create_pipeline(
                label,
                &no_clip_vertex,
                fragment,
                false,
                advanced_blend,
                hsl_blend,
                cull_mode,
                stencil.clone(),
                depth_compare,
                depth_write_enabled,
                color_write_mask,
                blend_enabled,
            );
            let unclipped_rect = clip_rect_vertex.as_ref().map(|vertex| {
                create_pipeline(
                    label,
                    vertex,
                    fragment,
                    true,
                    advanced_blend,
                    hsl_blend,
                    cull_mode,
                    stencil.clone(),
                    depth_compare,
                    depth_write_enabled,
                    color_write_mask,
                    blend_enabled,
                )
            });
            let path_clip = create_pipeline(
                label,
                &no_clip_vertex,
                fragment,
                false,
                advanced_blend,
                hsl_blend,
                cull_mode,
                path_clip_stencil.clone(),
                depth_compare,
                depth_write_enabled,
                color_write_mask,
                blend_enabled,
            );
            let path_clip_rect = clip_rect_vertex.as_ref().map(|vertex| {
                create_pipeline(
                    label,
                    vertex,
                    fragment,
                    true,
                    advanced_blend,
                    hsl_blend,
                    cull_mode,
                    path_clip_stencil,
                    depth_compare,
                    depth_write_enabled,
                    color_write_mask,
                    blend_enabled,
                )
            });
            PipelineVariants {
                unclipped,
                unclipped_rect,
                path_clip,
                path_clip_rect,
            }
        };
        let create_direct_pipelines = |fixed_label: &'static str,
                                       advanced_labels: Option<(&'static str, &'static str)>,
                                       cull_mode,
                                       stencil: wgpu::StencilState,
                                       path_clip_stencil: wgpu::StencilState,
                                       depth_compare,
                                       depth_write_enabled,
                                       color_write_mask| {
            let fixed = create_direct_variants(
                fixed_label,
                &fixed_fragment,
                false,
                false,
                cull_mode,
                stencil.clone(),
                path_clip_stencil.clone(),
                depth_compare,
                depth_write_enabled,
                color_write_mask,
                true,
            );
            let opaque_fixed = create_direct_variants(
                fixed_label,
                &fixed_fragment,
                false,
                false,
                cull_mode,
                stencil.clone(),
                path_clip_stencil.clone(),
                depth_compare,
                depth_write_enabled,
                color_write_mask,
                false,
            );
            let advanced = advanced_labels.map(|(advanced_label, _)| {
                create_direct_variants(
                    advanced_label,
                    &advanced_fragment,
                    true,
                    false,
                    cull_mode,
                    stencil.clone(),
                    path_clip_stencil.clone(),
                    depth_compare,
                    depth_write_enabled,
                    color_write_mask,
                    true,
                )
            });
            let advanced_hsl = advanced_labels.map(|(_, advanced_hsl_label)| {
                create_direct_variants(
                    advanced_hsl_label,
                    &advanced_fragment,
                    true,
                    true,
                    cull_mode,
                    stencil,
                    path_clip_stencil,
                    depth_compare,
                    depth_write_enabled,
                    color_write_mask,
                    true,
                )
            });
            DirectPipelineSet {
                fixed,
                opaque_fixed,
                advanced,
                advanced_hsl,
            }
        };
        let active_clip_face = stencil_face(wgpu::CompareFunction::Equal, keep, keep);
        let active_clip_stencil = stencil_state(active_clip_face, active_clip_face, 0xff, 0xff);
        // renderer/src/gpu.cpp::get_depth_state(msaaStrokes): one depth hit per
        // sample prevents a compound stroke from blending with itself.
        let stroke = create_direct_pipelines(
            "nuxie-msaa-stroke",
            Some((
                "nuxie-msaa-stroke-advanced",
                "nuxie-msaa-stroke-advanced-hsl",
            )),
            Some(wgpu::Face::Back),
            stencil_state(disabled_face, disabled_face, 0xff, 0xff),
            active_clip_stencil,
            wgpu::CompareFunction::Less,
            true,
            wgpu::ColorWrites::ALL,
        );
        // renderer/src/gpu.cpp: MSAA midpoint-fan fill pipeline states.
        let borrowed_face = stencil_face(
            wgpu::CompareFunction::Always,
            keep,
            wgpu::StencilOperation::IncrementWrap,
        );
        let clipped_borrowed_face = stencil_face(
            wgpu::CompareFunction::LessEqual,
            keep,
            wgpu::StencilOperation::IncrementWrap,
        );
        let fill_borrowed = create_direct_pipelines(
            "nuxie-msaa-fill-borrowed",
            None,
            Some(wgpu::Face::Front),
            stencil_state(borrowed_face, borrowed_face, 0xff, 0x7f),
            stencil_state(clipped_borrowed_face, clipped_borrowed_face, 0xff, 0x7f),
            wgpu::CompareFunction::Less,
            false,
            wgpu::ColorWrites::empty(),
        );
        let fill_front = stencil_face(
            wgpu::CompareFunction::Equal,
            wgpu::StencilOperation::DecrementClamp,
            keep,
        );
        let fill_back = stencil_face(
            wgpu::CompareFunction::Less,
            keep,
            wgpu::StencilOperation::Zero,
        );
        let fill_stencil = stencil_state(fill_front, fill_back, 0x7f, 0x7f);
        let clipped_fill_stencil = stencil_state(fill_front, fill_back, 0xff, 0x7f);
        let fill_forward = create_direct_pipelines(
            "nuxie-msaa-fill-forward",
            Some((
                "nuxie-msaa-fill-forward-advanced",
                "nuxie-msaa-fill-forward-advanced-hsl",
            )),
            Some(wgpu::Face::Back),
            fill_stencil.clone(),
            clipped_fill_stencil.clone(),
            wgpu::CompareFunction::Less,
            true,
            wgpu::ColorWrites::ALL,
        );
        let fill_cleanup = create_direct_pipelines(
            "nuxie-msaa-fill-cleanup",
            Some((
                "nuxie-msaa-fill-cleanup-advanced",
                "nuxie-msaa-fill-cleanup-advanced-hsl",
            )),
            Some(wgpu::Face::Front),
            fill_stencil.clone(),
            clipped_fill_stencil.clone(),
            wgpu::CompareFunction::Less,
            true,
            wgpu::ColorWrites::ALL,
        );
        let clockwise_fill_cleanup = create_direct_pipelines(
            "nuxie-msaa-clockwise-fill-cleanup",
            None,
            Some(wgpu::Face::Front),
            fill_stencil,
            clipped_fill_stencil,
            wgpu::CompareFunction::Less,
            false,
            wgpu::ColorWrites::empty(),
        );
        let even_odd_front = stencil_face(
            wgpu::CompareFunction::Always,
            keep,
            wgpu::StencilOperation::DecrementWrap,
        );
        let even_odd_back = stencil_face(
            wgpu::CompareFunction::Always,
            keep,
            wgpu::StencilOperation::IncrementWrap,
        );
        let clipped_even_odd_front = stencil_face(
            wgpu::CompareFunction::LessEqual,
            keep,
            wgpu::StencilOperation::DecrementWrap,
        );
        let clipped_even_odd_back = stencil_face(
            wgpu::CompareFunction::LessEqual,
            keep,
            wgpu::StencilOperation::IncrementWrap,
        );
        let even_odd_fill_stencil = create_direct_pipelines(
            "nuxie-msaa-even-odd-fill-stencil",
            None,
            None,
            stencil_state(even_odd_front, even_odd_back, 0xff, 0x01),
            stencil_state(clipped_even_odd_front, clipped_even_odd_back, 0xff, 0x01),
            wgpu::CompareFunction::Less,
            false,
            wgpu::ColorWrites::empty(),
        );
        let even_odd_cover_face = stencil_face(
            wgpu::CompareFunction::NotEqual,
            keep,
            wgpu::StencilOperation::Zero,
        );
        let even_odd_fill_cover = create_direct_pipelines(
            "nuxie-msaa-even-odd-fill-cover",
            Some((
                "nuxie-msaa-even-odd-fill-cover-advanced",
                "nuxie-msaa-even-odd-fill-cover-advanced-hsl",
            )),
            None,
            stencil_state(even_odd_cover_face, even_odd_cover_face, 0x7f, 0x01),
            stencil_state(even_odd_cover_face, even_odd_cover_face, 0x7f, 0x01),
            wgpu::CompareFunction::Less,
            true,
            wgpu::ColorWrites::ALL,
        );
        let clip_borrowed_face = stencil_face(
            wgpu::CompareFunction::Always,
            keep,
            wgpu::StencilOperation::IncrementWrap,
        );
        let clip_borrowed_pipeline = create_pipeline(
            "nuxie-msaa-path-clip-borrowed-pipeline",
            &no_clip_vertex,
            &fixed_fragment,
            false,
            false,
            false,
            Some(wgpu::Face::Front),
            stencil_state(clip_borrowed_face, clip_borrowed_face, 0xff, 0x7f),
            wgpu::CompareFunction::Less,
            false,
            wgpu::ColorWrites::empty(),
            false,
        );
        let clip_front = stencil_face(
            wgpu::CompareFunction::Equal,
            wgpu::StencilOperation::DecrementClamp,
            wgpu::StencilOperation::Replace,
        );
        let clip_back = stencil_face(
            wgpu::CompareFunction::Less,
            keep,
            wgpu::StencilOperation::Replace,
        );
        let clip_update_pipeline = create_pipeline(
            "nuxie-msaa-path-clip-update-pipeline",
            &no_clip_vertex,
            &fixed_fragment,
            false,
            false,
            false,
            Some(wgpu::Face::Back),
            stencil_state(clip_front, clip_back, 0x7f, 0xff),
            wgpu::CompareFunction::Less,
            false,
            wgpu::ColorWrites::empty(),
            false,
        );
        let clip_cleanup_pipeline = create_pipeline(
            "nuxie-msaa-path-clip-cleanup-pipeline",
            &no_clip_vertex,
            &fixed_fragment,
            false,
            false,
            false,
            Some(wgpu::Face::Front),
            stencil_state(clip_front, clip_back, 0x7f, 0xff),
            wgpu::CompareFunction::Less,
            false,
            wgpu::ColorWrites::empty(),
            false,
        );
        let clockwise_clip_cleanup_pipeline = create_pipeline(
            "nuxie-msaa-path-clockwise-clip-cleanup-pipeline",
            &no_clip_vertex,
            &fixed_fragment,
            false,
            false,
            false,
            Some(wgpu::Face::Front),
            stencil_state(clip_front, clip_back, 0x7f, 0x7f),
            wgpu::CompareFunction::Less,
            false,
            wgpu::ColorWrites::empty(),
            false,
        );
        let even_odd_clip_front = stencil_face(
            wgpu::CompareFunction::Always,
            keep,
            wgpu::StencilOperation::DecrementWrap,
        );
        let even_odd_clip_back = stencil_face(
            wgpu::CompareFunction::Always,
            keep,
            wgpu::StencilOperation::IncrementWrap,
        );
        let even_odd_clip_stencil_pipeline = create_pipeline(
            "nuxie-msaa-path-even-odd-clip-stencil-pipeline",
            &no_clip_vertex,
            &fixed_fragment,
            false,
            false,
            false,
            None,
            stencil_state(even_odd_clip_front, even_odd_clip_back, 0xff, 0x01),
            wgpu::CompareFunction::Less,
            false,
            wgpu::ColorWrites::empty(),
            false,
        );
        let even_odd_cover_face = stencil_face(
            wgpu::CompareFunction::NotEqual,
            keep,
            wgpu::StencilOperation::Replace,
        );
        let even_odd_clip_cover_pipeline = create_pipeline(
            "nuxie-msaa-path-even-odd-clip-cover-pipeline",
            &no_clip_vertex,
            &fixed_fragment,
            false,
            false,
            false,
            None,
            stencil_state(even_odd_cover_face, even_odd_cover_face, 0x7f, 0xff),
            wgpu::CompareFunction::Less,
            false,
            wgpu::ColorWrites::empty(),
            false,
        );
        let nested_front = stencil_face(
            wgpu::CompareFunction::LessEqual,
            keep,
            wgpu::StencilOperation::DecrementWrap,
        );
        let nested_back = stencil_face(
            wgpu::CompareFunction::LessEqual,
            keep,
            wgpu::StencilOperation::IncrementWrap,
        );
        let nested_clip_pipeline = create_pipeline(
            "nuxie-msaa-path-nested-clip-pipeline",
            &no_clip_vertex,
            &fixed_fragment,
            false,
            false,
            false,
            None,
            stencil_state(nested_front, nested_back, 0xff, 0x7f),
            wgpu::CompareFunction::Less,
            false,
            wgpu::ColorWrites::empty(),
            false,
        );
        let nested_even_odd_clip_pipeline = create_pipeline(
            "nuxie-msaa-path-nested-even-odd-clip-pipeline",
            &no_clip_vertex,
            &fixed_fragment,
            false,
            false,
            false,
            None,
            stencil_state(nested_front, nested_back, 0xff, 0x01),
            wgpu::CompareFunction::Less,
            false,
            wgpu::ColorWrites::empty(),
            false,
        );
        let dummy_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-msaa-path-dummy-texture"),
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
        let dummy_view = dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());
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
            label: Some("nuxie-msaa-path-dummy-image-group"),
            layout: &image_layout,
            entries: &[
                binding(12, wgpu::BindingResource::TextureView(&dummy_view)),
                binding(14, wgpu::BindingResource::Sampler(&image_samplers[0])),
            ],
        });
        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nuxie-msaa-path-linear-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let sampler_group = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-msaa-path-sampler-group"),
            layout: &sampler_layout,
            entries: &[
                binding(9, wgpu::BindingResource::Sampler(&linear_sampler)),
                binding(10, wgpu::BindingResource::Sampler(&linear_sampler)),
            ],
        });
        Self {
            stroke,
            fill_borrowed,
            fill_forward,
            fill_cleanup,
            clockwise_fill_cleanup,
            even_odd_fill_stencil,
            even_odd_fill_cover,
            clip_borrowed_pipeline,
            clip_update_pipeline,
            clip_cleanup_pipeline,
            clockwise_clip_cleanup_pipeline,
            even_odd_clip_stencil_pipeline,
            even_odd_clip_cover_pipeline,
            nested_clip_pipeline,
            nested_even_odd_clip_pipeline,
            flush_layout,
            image_layout,
            _dummy_texture: dummy_texture,
            dummy_view,
            dummy_image_group,
            image_samplers,
            sampler_group,
        }
    }

    pub(crate) fn supports_clip_rect(&self) -> bool {
        self.stroke.fixed.unclipped_rect.is_some()
    }

    pub(crate) fn direct_pipeline(
        &self,
        kind: DirectPathPipelineKind,
        path_clip: bool,
        clip_rect: bool,
        opaque: bool,
        advanced_blend: bool,
        hsl_blend: bool,
    ) -> &wgpu::RenderPipeline {
        let pipelines = match kind {
            DirectPathPipelineKind::Stroke => &self.stroke,
            DirectPathPipelineKind::FillBorrowed => &self.fill_borrowed,
            DirectPathPipelineKind::FillForward => &self.fill_forward,
            DirectPathPipelineKind::FillCleanup => &self.fill_cleanup,
            DirectPathPipelineKind::ClockwiseFillCleanup => &self.clockwise_fill_cleanup,
            DirectPathPipelineKind::EvenOddFillStencil => &self.even_odd_fill_stencil,
            DirectPathPipelineKind::EvenOddFillCover => &self.even_odd_fill_cover,
        };
        let variants = if advanced_blend {
            if hsl_blend {
                pipelines.advanced_hsl.as_ref()
            } else {
                pipelines.advanced.as_ref()
            }
            .unwrap_or(&pipelines.fixed)
        } else if opaque {
            &pipelines.opaque_fixed
        } else {
            &pipelines.fixed
        };
        match (path_clip, clip_rect) {
            (false, false) => &variants.unclipped,
            (false, true) => variants
                .unclipped_rect
                .as_ref()
                .expect("clip-rect path draw prepared without clip-distance pipeline"),
            (true, false) => &variants.path_clip,
            (true, true) => variants
                .path_clip_rect
                .as_ref()
                .expect("path-and-rect-clipped draw prepared without clip-distance pipeline"),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare_resources(
        &self,
        device: &wgpu::Device,
        uploads: &mut TessellationUploadFrame<'_>,
        tessellation_view: &wgpu::TextureView,
        feather_lut: &wgpu::TextureView,
        gradient: Option<&wgpu::TextureView>,
        destination: Option<&wgpu::TextureView>,
        uniforms: &FlushUniforms,
        paths: &[PathData],
        paints: &[PaintData],
        paint_aux: &[PaintAuxData],
        contours: &[ContourData],
    ) -> PreparedPathResources {
        // C++ maps these resource rings flush-wide. Exact aligned slices in
        // the guarded frame arena give wgpu the same completed-frame lifetime.
        let uniform_buffer = uploads.upload_uniforms(device, bytemuck::bytes_of(uniforms));
        let path_buffer = uploads.upload_storage(device, bytemuck::cast_slice(paths));
        let paint_buffer = uploads.upload_storage(device, bytemuck::cast_slice(paints));
        let paint_aux_buffer = uploads.upload_storage(device, bytemuck::cast_slice(paint_aux));
        let contour_buffer = uploads.upload_storage(device, bytemuck::cast_slice(contours));
        let flush_group = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-msaa-path-flush-group"),
            layout: &self.flush_layout,
            entries: &[
                binding(0, uniform_buffer.binding()),
                binding(3, path_buffer.binding()),
                binding(4, paint_buffer.binding()),
                binding(5, paint_aux_buffer.binding()),
                binding(6, contour_buffer.binding()),
                binding(8, wgpu::BindingResource::TextureView(tessellation_view)),
                binding(
                    9,
                    wgpu::BindingResource::TextureView(gradient.unwrap_or(&self.dummy_view)),
                ),
                binding(10, wgpu::BindingResource::TextureView(feather_lut)),
                binding(
                    13,
                    wgpu::BindingResource::TextureView(destination.unwrap_or(&self.dummy_view)),
                ),
            ],
        });
        PreparedPathResources { flush_group }
    }

    pub(crate) fn prepare_draw(
        &self,
        device: &wgpu::Device,
        resources: &PreparedPathResources,
        image: Option<(&wgpu::TextureView, ImageSampler)>,
        base_instance: u32,
        instance_count: u32,
    ) -> PreparedPathDraw {
        let has_image = image.is_some();
        let image_group = image.map_or_else(
            || self.dummy_image_group.clone(),
            |(view, sampler)| {
                device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("nuxie-msaa-path-image-group"),
                    layout: &self.image_layout,
                    entries: &[
                        binding(12, wgpu::BindingResource::TextureView(view)),
                        binding(
                            14,
                            wgpu::BindingResource::Sampler(
                                &self.image_samplers[sampler.as_key() as usize],
                            ),
                        ),
                    ],
                })
            },
        );
        PreparedPathDraw {
            flush_group: resources.flush_group.clone(),
            image_group,
            sampler_group: self.sampler_group.clone(),
            base_instance,
            instance_count,
            has_image,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::FIXED_COLOR_FRAGMENT_CONSTANTS;

    #[test]
    fn fixed_color_paths_enable_cpp_dither_shader_feature() {
        assert_eq!(FIXED_COLOR_FRAGMENT_CONSTANTS, [("2", 0.0), ("7", 1.0)]);
    }
}
