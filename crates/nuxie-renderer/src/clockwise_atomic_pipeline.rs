//! Clockwise-atomic fill passes translated from Rive's Vulkan shader family.
//!
//! This is intentionally separate from `atomic_pipeline`: the two shader
//! families use incompatible coverage-buffer encodings and pass schedules.

use crate::gpu::{FlushUniforms, PaintAuxData, PaintData, PatchVertex, TriangleVertex};
use crate::tessellator::{FrameUploadPayload, TessellationFlushResources, TessellationUploadFrame};
use crate::work_metrics::{CountedCommandEncoderExt, CountedDeviceExt};
use std::{
    num::NonZeroU64,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, MutexGuard,
    },
};

const CLOCKWISE_ATOMIC_BACKING_RING_SIZE: usize = 3;
const CLOCKWISE_COVERAGE_PREFIX_STEP: u32 = 1 << 20;

fn rotate_coverage_prefix(prefix: &mut u32) -> bool {
    let mut needs_clear = false;
    loop {
        if *prefix == 0 {
            needs_clear = true;
        }
        *prefix = prefix.wrapping_add(CLOCKWISE_COVERAGE_PREFIX_STEP);
        if *prefix != 0 {
            return needs_clear;
        }
    }
}

fn retained_coverage_capacity_words(required_words: u64, max_words: u64) -> u64 {
    assert!(required_words != 0 && required_words <= max_words);
    required_words
        .saturating_add(required_words / 4)
        .min(max_words)
}

fn canonicalize_coverage_word(word: u32, prefix: u32) -> u32 {
    if word < prefix {
        return 0;
    }
    debug_assert_eq!(
        word & !(CLOCKWISE_COVERAGE_PREFIX_STEP - 1),
        prefix,
        "CWA coverage word belongs to a future generation"
    );
    CLOCKWISE_COVERAGE_PREFIX_STEP | (word & (CLOCKWISE_COVERAGE_PREFIX_STEP - 1))
}

pub(crate) struct ClockwiseAtomicPipeline {
    borrowed_path: wgpu::RenderPipeline,
    borrowed_interior: wgpu::RenderPipeline,
    path: wgpu::RenderPipeline,
    path_clip_rect: wgpu::RenderPipeline,
    interior: wgpu::RenderPipeline,
    interior_clip_rect: wgpu::RenderPipeline,
    clipped_path: wgpu::RenderPipeline,
    clipped_path_clip_rect: wgpu::RenderPipeline,
    clipped_interior: wgpu::RenderPipeline,
    clipped_interior_clip_rect: wgpu::RenderPipeline,
    outer_clip_path: wgpu::RenderPipeline,
    outer_clip_interior: wgpu::RenderPipeline,
    nested_clip_path: wgpu::RenderPipeline,
    nested_clip_interior: wgpu::RenderPipeline,
    flush_layout: wgpu::BindGroupLayout,
    sampled_clip_layout: wgpu::BindGroupLayout,
    _dummy_texture: wgpu::Texture,
    dummy_view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
    image_group: wgpu::BindGroup,
    _scratch_clip_buffer: wgpu::Buffer,
    clip_group: wgpu::BindGroup,
    dummy_sampled_clip_group: wgpu::BindGroup,
    sampler_group: wgpu::BindGroup,
    backing_slots: [Mutex<ClockwiseAtomicBackingSlot>; CLOCKWISE_ATOMIC_BACKING_RING_SIZE],
    next_backing_slot: AtomicUsize,
}

#[derive(Default)]
struct ClockwiseAtomicBackingSlot {
    coverage: Option<RetainedCoverageBuffer>,
    clip_texture: Option<RetainedClipTexture>,
}

struct RetainedCoverageBuffer {
    buffer: wgpu::Buffer,
    capacity_words: u64,
    prefix: u32,
}

struct RetainedClipTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampled_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

pub(crate) struct ClockwiseAtomicBackingFrame<'a> {
    slot: MutexGuard<'a, ClockwiseAtomicBackingSlot>,
    clear_recorded_unsubmitted: bool,
}

pub(crate) struct ClockwiseAtomicCoverageRun {
    buffer: wgpu::Buffer,
    size: NonZeroU64,
    word_count: usize,
    prefix: u32,
}

impl ClockwiseAtomicBackingSlot {
    fn prepare_clip_texture(
        &mut self,
        device: &wgpu::Device,
        sampled_clip_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::BindGroup) {
        if self
            .clip_texture
            .as_ref()
            .is_none_or(|clip| clip.width != width || clip.height != height)
        {
            self.clip_texture = Some(create_clip_texture(
                device,
                sampled_clip_layout,
                width,
                height,
            ));
        }
        let clip = self
            .clip_texture
            .as_ref()
            .expect("CWA clip texture was initialized");
        (
            clip.texture.clone(),
            clip.view.clone(),
            clip.sampled_group.clone(),
        )
    }
}

impl ClockwiseAtomicBackingFrame<'_> {
    pub(crate) fn prepare_coverage(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        word_count: usize,
    ) -> ClockwiseAtomicCoverageRun {
        let word_count = word_count.max(1);
        let required_words = u64::try_from(word_count).expect("CWA coverage word count fits u64");
        let limits = device.limits();
        let max_words = limits
            .max_storage_buffer_binding_size
            .min(limits.max_buffer_size)
            / std::mem::size_of::<u32>() as u64;
        assert!(
            required_words <= max_words,
            "CWA coverage exceeds the device storage-buffer limit"
        );
        if self
            .slot
            .coverage
            .as_ref()
            .is_none_or(|coverage| coverage.capacity_words < required_words)
        {
            let capacity_words = retained_coverage_capacity_words(required_words, max_words);
            let size = capacity_words
                .checked_mul(std::mem::size_of::<u32>() as u64)
                .expect("CWA retained coverage byte size overflow");
            self.slot.coverage = Some(RetainedCoverageBuffer {
                buffer: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("nuxie-cwa-retained-coverage"),
                    size,
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_SRC
                        | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                capacity_words,
                prefix: 0,
            });
        }
        let coverage = self
            .slot
            .coverage
            .as_mut()
            .expect("CWA retained coverage was initialized");
        if rotate_coverage_prefix(&mut coverage.prefix) {
            // Match C++: allocation, growth, and the rare 12-bit generation
            // wrap clear the complete retained allocation. Recording this in
            // the frame encoder keeps it in the same physical command buffer.
            self.clear_recorded_unsubmitted = true;
            encoder.clear_counted_buffer(&coverage.buffer, 0, None);
        }
        let size = required_words
            .checked_mul(std::mem::size_of::<u32>() as u64)
            .and_then(NonZeroU64::new)
            .expect("nonempty CWA coverage byte size");
        ClockwiseAtomicCoverageRun {
            buffer: coverage.buffer.clone(),
            size,
            word_count,
            prefix: coverage.prefix,
        }
    }

    pub(crate) fn did_submit(&mut self) {
        self.clear_recorded_unsubmitted = false;
    }
}

impl Drop for ClockwiseAtomicBackingFrame<'_> {
    fn drop(&mut self) {
        if self.clear_recorded_unsubmitted {
            // The clear lived in an encoder that was abandoned. Returning to
            // prefix zero makes the clear requirement sticky for the next
            // lease instead of trusting commands that never reached the GPU.
            if let Some(coverage) = &mut self.slot.coverage {
                coverage.prefix = 0;
            }
        }
    }
}

impl ClockwiseAtomicCoverageRun {
    pub(crate) fn prefix(&self) -> u32 {
        self.prefix
    }

    fn binding(&self) -> wgpu::BindingResource<'_> {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &self.buffer,
            offset: 0,
            size: Some(self.size),
        })
    }

    fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::Buffer) {
        encoder.copy_buffer_to_buffer(&self.buffer, 0, target, 0, self.size.get());
    }
}

fn create_clip_texture(
    device: &wgpu::Device,
    sampled_clip_layout: &wgpu::BindGroupLayout,
    width: u32,
    height: u32,
) -> RetainedClipTexture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("nuxie-cwa-clip-texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let sampled_group = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("nuxie-cwa-sampled-clip-group"),
        layout: sampled_clip_layout,
        entries: &[binding(1, wgpu::BindingResource::TextureView(&view))],
    });
    RetainedClipTexture {
        texture,
        view,
        sampled_group,
        width,
        height,
    }
}

pub(crate) struct ClockwiseAtomicDraw<'a> {
    pub tessellation: &'a wgpu::TextureView,
    pub borrowed_base_instance: u32,
    pub main_base_instance: u32,
    pub instance_count: u32,
    pub patch_index_range: std::ops::Range<u32>,
    pub borrowed_triangles: &'a [TriangleVertex],
    pub main_triangles: &'a [TriangleVertex],
    pub main_triangle_batches: &'a [ClockwiseAtomicTriangleBatch],
    pub kind: ClockwiseAtomicDrawKind,
    pub has_clip_rect: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ClockwiseAtomicTriangleBatch {
    pub vertex_start: u32,
    pub vertex_count: u32,
    pub instance_count: u32,
}

pub(crate) struct ClockwiseAtomicCoverageReadback {
    pub borrowed: wgpu::Buffer,
    pub main: wgpu::Buffer,
    pub word_count: usize,
    pub prefix: u32,
    pub clip_updates: Vec<wgpu::Buffer>,
    pub clip_bytes_per_row: u32,
    pub clip_height: u32,
}

impl ClockwiseAtomicCoverageReadback {
    pub(crate) fn canonicalize_words(&self, mut words: Vec<u32>) -> Vec<u32> {
        for word in &mut words {
            *word = canonicalize_coverage_word(*word, self.prefix);
        }
        words
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClockwiseAtomicDrawKind {
    Content,
    ClippedContent,
    OutermostClip,
    NestedClip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClockwiseAtomicMainPassKind {
    Target,
    ClipLoad,
    ClipClear,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ClockwiseAtomicMainPass {
    kind: ClockwiseAtomicMainPassKind,
    draw_range: std::ops::Range<usize>,
}

fn clockwise_atomic_main_passes(
    kinds: &[ClockwiseAtomicDrawKind],
    capture_clip_updates: bool,
) -> Vec<ClockwiseAtomicMainPass> {
    let is_clip = |kind| {
        matches!(
            kind,
            ClockwiseAtomicDrawKind::OutermostClip | ClockwiseAtomicDrawKind::NestedClip
        )
    };
    let pass_kind = |kind| match kind {
        ClockwiseAtomicDrawKind::Content | ClockwiseAtomicDrawKind::ClippedContent => {
            ClockwiseAtomicMainPassKind::Target
        }
        ClockwiseAtomicDrawKind::OutermostClip => ClockwiseAtomicMainPassKind::ClipClear,
        ClockwiseAtomicDrawKind::NestedClip => ClockwiseAtomicMainPassKind::ClipLoad,
    };

    let mut passes = Vec::new();
    let mut start = 0;
    for end in 1..=kinds.len() {
        let boundary = end == kinds.len()
            || capture_clip_updates && kinds[end - 1] == ClockwiseAtomicDrawKind::NestedClip
            || kinds[end] == ClockwiseAtomicDrawKind::OutermostClip
            || is_clip(kinds[end - 1]) != is_clip(kinds[end]);
        if boundary {
            passes.push(ClockwiseAtomicMainPass {
                kind: pass_kind(kinds[start]),
                draw_range: start..end,
            });
            start = end;
        }
    }
    passes
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
                storage_entry(2, true),
                storage_entry(3, true),
                storage_entry(4, true),
                storage_entry(5, true),
                storage_entry(6, false),
                texture_entry(7, wgpu::TextureSampleType::Uint),
                texture_entry(8, wgpu::TextureSampleType::Float { filterable: true }),
                texture_entry(9, wgpu::TextureSampleType::Float { filterable: true }),
                texture_entry(10, wgpu::TextureSampleType::Float { filterable: true }),
            ],
        });
        let image_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-cwa-image-layout"),
            entries: &[
                texture_entry(11, wgpu::TextureSampleType::Float { filterable: true }),
                sampler_entry(13),
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
            entries: &[sampler_entry(8), sampler_entry(9), sampler_entry(10)],
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
        let path_clip_rect = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-cwa-path-clip-rect-pipeline"),
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
                compilation_options: options(&[("0", 0.0), ("1", 1.0), ("3", 0.0), ("7", 0.0)]),
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
            primitive: cull_counterclockwise(),
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
        let interior_clip_rect = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-cwa-interior-clip-rect-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &interior_vertex,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(TriangleVertex::layout())],
            },
            primitive: cull_counterclockwise(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &interior_fragment,
                entry_point: Some("main"),
                compilation_options: options(&[("0", 0.0), ("1", 1.0), ("7", 0.0)]),
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
            primitive: cull_counterclockwise(),
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
            primitive: cull_counterclockwise(),
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
        let clipped_path_clip_rect =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("nuxie-cwa-clipped-path-clip-rect-pipeline"),
                layout: Some(&sampled_clip_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &path_vertex,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(PatchVertex::layout())],
                },
                primitive: cull_counterclockwise(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &sampled_clip_path_fragment,
                    entry_point: Some("main"),
                    compilation_options: options(&[("0", 1.0), ("1", 1.0), ("3", 0.0), ("7", 0.0)]),
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
            primitive: cull_counterclockwise(),
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
        let clipped_interior_clip_rect =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("nuxie-cwa-clipped-interior-clip-rect-pipeline"),
                layout: Some(&sampled_clip_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &interior_vertex,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(TriangleVertex::layout())],
                },
                primitive: cull_counterclockwise(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &sampled_clip_interior_fragment,
                    entry_point: Some("main"),
                    compilation_options: options(&[("0", 1.0), ("1", 1.0), ("7", 0.0)]),
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
        // These are the CWA equivalents of C++'s retained null texture and
        // image-sampler tables. They never depend on a flush, so recreating
        // them in encode_fills only adds driver allocation and bind-group work.
        let dummy_texture = device.create_texture(&wgpu::TextureDescriptor {
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
        let dummy_view = dummy_texture.create_view(&Default::default());
        let sampler = device.create_sampler(&linear_sampler());
        let image_group = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-cwa-image-group"),
            layout: &image_layout,
            entries: &[
                binding(11, wgpu::BindingResource::TextureView(&dummy_view)),
                binding(13, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        // ENABLE_CLIPPING is specialized false for every pipeline that uses
        // the storage-buffer clip layout. The translated fixed-color shaders
        // retain an unconditional zero store, but never read this plane. A
        // one-word robust-access sink therefore matches C++'s null-resource
        // binding and avoids allocating and touching a full viewport plane.
        let scratch_clip_buffer =
            device.create_counted_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("nuxie-cwa-scratch-clip"),
                contents: bytemuck::bytes_of(&0u32),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let clip_group = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-cwa-clip-group"),
            layout: &clip_layout,
            entries: &[binding(1, scratch_clip_buffer.as_entire_binding())],
        });
        let dummy_sampled_clip_group =
            device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("nuxie-cwa-dummy-sampled-clip-group"),
                layout: &sampled_clip_layout,
                entries: &[binding(1, wgpu::BindingResource::TextureView(&dummy_view))],
            });
        let sampler_group = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-cwa-sampler-group"),
            layout: &sampler_layout,
            entries: &[
                binding(8, wgpu::BindingResource::Sampler(&sampler)),
                binding(9, wgpu::BindingResource::Sampler(&sampler)),
                binding(10, wgpu::BindingResource::Sampler(&sampler)),
            ],
        });
        Self {
            borrowed_path,
            borrowed_interior,
            path,
            path_clip_rect,
            interior,
            interior_clip_rect,
            clipped_path,
            clipped_path_clip_rect,
            clipped_interior,
            clipped_interior_clip_rect,
            outer_clip_path,
            outer_clip_interior,
            nested_clip_path,
            nested_clip_interior,
            flush_layout,
            sampled_clip_layout,
            _dummy_texture: dummy_texture,
            dummy_view,
            _sampler: sampler,
            image_group,
            _scratch_clip_buffer: scratch_clip_buffer,
            clip_group,
            dummy_sampled_clip_group,
            sampler_group,
            backing_slots: std::array::from_fn(|_| {
                Mutex::new(ClockwiseAtomicBackingSlot::default())
            }),
            next_backing_slot: AtomicUsize::new(0),
        }
    }

    pub(crate) fn begin_frame_backing(&self) -> ClockwiseAtomicBackingFrame<'_> {
        let slot_index = self.next_backing_slot.fetch_add(1, Ordering::Relaxed)
            % CLOCKWISE_ATOMIC_BACKING_RING_SIZE;
        let slot = self.backing_slots[slot_index]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        ClockwiseAtomicBackingFrame {
            slot,
            clear_recorded_unsubmitted: false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_fills(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        feather_lut: &wgpu::TextureView,
        gradient: Option<&wgpu::TextureView>,
        patch_vertices: &wgpu::Buffer,
        patch_indices: &wgpu::Buffer,
        draws: &[ClockwiseAtomicDraw<'_>],
        uniforms: &FlushUniforms,
        flush_resources: &TessellationFlushResources,
        backing: &mut ClockwiseAtomicBackingFrame<'_>,
        coverage: &ClockwiseAtomicCoverageRun,
        uploads: &mut TessellationUploadFrame<'_>,
        paints: &[PaintData],
        paint_aux: &[PaintAuxData],
        capture_coverage: bool,
    ) -> Option<ClockwiseAtomicCoverageReadback> {
        assert!(!draws.is_empty());
        assert_eq!(
            uniforms.coverage_buffer_prefix,
            coverage.prefix(),
            "CWA coverage prefix must be chosen before tessellation uniforms are uploaded"
        );
        // C++ maps paints, paint auxiliaries, and triangle vertices from
        // retained frame rings. Pack the compatible read-only/vertex slices
        // into one arena copy. Coverage is intentionally kept on a separate
        // retained page because read-write storage usage is exclusive in
        // wgpu, even when buffer ranges do not overlap.
        let mut payloads = Vec::with_capacity(2 + draws.len() * 2);
        payloads.push(FrameUploadPayload::Storage(bytemuck::cast_slice(paints)));
        payloads.push(FrameUploadPayload::Storage(bytemuck::cast_slice(paint_aux)));
        for draw in draws {
            if !draw.borrowed_triangles.is_empty() {
                payloads.push(FrameUploadPayload::Vertex(bytemuck::cast_slice(
                    draw.borrowed_triangles,
                )));
            }
        }
        let main_triangle_upload_start = payloads.len();
        for draw in draws {
            if !draw.main_triangles.is_empty() {
                payloads.push(FrameUploadPayload::Vertex(bytemuck::cast_slice(
                    draw.main_triangles,
                )));
            }
        }
        let grouped_uploads = uploads.upload_group(device, encoder, &payloads);
        let paints = &grouped_uploads[0];
        let paint_aux = &grouped_uploads[1];
        let coverage_readback = capture_coverage.then(|| {
            let make_buffer = |label| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label),
                    size: coverage.size.get(),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                })
            };
            ClockwiseAtomicCoverageReadback {
                borrowed: make_buffer("nuxie-cwa-borrowed-coverage-readback"),
                main: make_buffer("nuxie-cwa-main-coverage-readback"),
                word_count: coverage.word_count,
                prefix: coverage.prefix,
                clip_updates: Vec::new(),
                clip_bytes_per_row: 0,
                clip_height: 0,
            }
        });
        let first_texture_clip_kind = draws
            .iter()
            .map(|draw| draw.kind)
            .find(|kind| *kind != ClockwiseAtomicDrawKind::Content);
        let clip_resources = first_texture_clip_kind.map(|first_kind| {
            if first_kind == ClockwiseAtomicDrawKind::OutermostClip {
                // The first clip pass clears the attachment, making reuse exact.
                backing.slot.prepare_clip_texture(
                    device,
                    &self.sampled_clip_layout,
                    uniforms.render_target_width,
                    uniforms.render_target_height,
                )
            } else {
                // Preserve the old fresh-texture semantics for unusual runs
                // that begin by loading or sampling clip state.
                let clip = create_clip_texture(
                    device,
                    &self.sampled_clip_layout,
                    uniforms.render_target_width,
                    uniforms.render_target_height,
                );
                (clip.texture, clip.view, clip.sampled_group)
            }
        });
        let clip = &self.clip_group;
        let clip_texture = clip_resources.as_ref().map(|(texture, _, _)| texture);
        let clip_view = clip_resources
            .as_ref()
            .map_or(&self.dummy_view, |(_, view, _)| view);
        let sampled_clip = clip_resources
            .as_ref()
            .map_or(&self.dummy_sampled_clip_group, |(_, _, group)| group);
        let flush_groups = draws
            .iter()
            .map(|draw| {
                device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("nuxie-cwa-flush-group"),
                    layout: &self.flush_layout,
                    entries: &[
                        binding(0, flush_resources.uniform_binding()),
                        binding(2, flush_resources.path_binding()),
                        binding(3, paints.binding()),
                        binding(4, paint_aux.binding()),
                        binding(5, flush_resources.contour_binding()),
                        binding(6, coverage.binding()),
                        binding(7, wgpu::BindingResource::TextureView(draw.tessellation)),
                        binding(
                            8,
                            wgpu::BindingResource::TextureView(
                                gradient.unwrap_or(&self.dummy_view),
                            ),
                        ),
                        binding(9, wgpu::BindingResource::TextureView(feather_lut)),
                        binding(10, wgpu::BindingResource::TextureView(&self.dummy_view)),
                    ],
                })
            })
            .collect::<Vec<_>>();
        let image = &self.image_group;
        let samplers = &self.sampler_group;
        let clip_bytes_per_row = (uniforms.render_target_width * 4)
            .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let mut clip_update_readbacks = Vec::new();

        // Path patches and interior triangles update private coverage ranges
        // atomically. All borrowed draws can therefore share one pass; the
        // required synchronization boundary is borrowed -> main.
        if draws
            .iter()
            .any(|draw| draw.kind != ClockwiseAtomicDrawKind::OutermostClip)
        {
            let attachments = [color_attachment(target)];
            let mut pass = encoder.begin_counted_render_pass(&render_pass_descriptor(
                "nuxie-cwa-borrowed-pass",
                &attachments,
            ));
            pass.set_index_buffer(patch_indices.slice(..), wgpu::IndexFormat::Uint16);
            let mut next_triangle_upload = 2;
            for (index, draw) in draws.iter().enumerate() {
                let triangle_upload = (!draw.borrowed_triangles.is_empty()).then(|| {
                    let upload = &grouped_uploads[next_triangle_upload];
                    next_triangle_upload += 1;
                    upload
                });
                if draw.kind == ClockwiseAtomicDrawKind::OutermostClip {
                    continue;
                }
                pass.set_pipeline(&self.borrowed_path);
                set_groups(&mut pass, &flush_groups[index], image, clip, samplers);
                pass.set_vertex_buffer(0, patch_vertices.slice(..));
                pass.draw_path_patches(
                    draw.patch_index_range.clone(),
                    0,
                    draw.borrowed_base_instance..draw.borrowed_base_instance + draw.instance_count,
                );
                if let Some(buffer) = triangle_upload {
                    pass.set_pipeline(&self.borrowed_interior);
                    set_groups(&mut pass, &flush_groups[index], image, clip, samplers);
                    pass.set_vertex_buffer(0, buffer.slice());
                    pass.draw(0..draw.borrowed_triangles.len() as u32, 0..1);
                }
            }
            debug_assert_eq!(next_triangle_upload, main_triangle_upload_start);
            drop(pass);
        }

        if let Some(readback) = &coverage_readback {
            coverage.copy_to_buffer(encoder, &readback.borrowed);
        }

        let main_kinds = draws.iter().map(|draw| draw.kind).collect::<Vec<_>>();
        let mut next_triangle_upload = main_triangle_upload_start;
        for main_pass in clockwise_atomic_main_passes(&main_kinds, capture_coverage) {
            let (label, clip_load) = match main_pass.kind {
                ClockwiseAtomicMainPassKind::Target => ("nuxie-cwa-main-pass", None),
                ClockwiseAtomicMainPassKind::ClipLoad => {
                    ("nuxie-cwa-clip-pass", Some(wgpu::LoadOp::Load))
                }
                ClockwiseAtomicMainPassKind::ClipClear => (
                    "nuxie-cwa-clip-pass",
                    Some(wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)),
                ),
            };
            let mut attachments = vec![color_attachment(target)];
            if let Some(load) = clip_load {
                attachments.push(color_attachment_with_load(clip_view, load));
            }
            let mut pass =
                encoder.begin_counted_render_pass(&render_pass_descriptor(label, &attachments));
            pass.set_index_buffer(patch_indices.slice(..), wgpu::IndexFormat::Uint16);

            for index in main_pass.draw_range.clone() {
                let draw = &draws[index];
                let triangle_upload = (!draw.main_triangles.is_empty()).then(|| {
                    let upload = &grouped_uploads[next_triangle_upload];
                    next_triangle_upload += 1;
                    upload
                });
                match draw.kind {
                    ClockwiseAtomicDrawKind::Content | ClockwiseAtomicDrawKind::ClippedContent => {
                        debug_assert_eq!(main_pass.kind, ClockwiseAtomicMainPassKind::Target);
                        pass.set_pipeline(match (draw.kind, draw.has_clip_rect) {
                            (ClockwiseAtomicDrawKind::ClippedContent, true) => {
                                &self.clipped_path_clip_rect
                            }
                            (ClockwiseAtomicDrawKind::ClippedContent, false) => &self.clipped_path,
                            (_, true) => &self.path_clip_rect,
                            (_, false) => &self.path,
                        });
                        if draw.kind == ClockwiseAtomicDrawKind::ClippedContent {
                            set_groups(
                                &mut pass,
                                &flush_groups[index],
                                image,
                                sampled_clip,
                                samplers,
                            );
                        } else {
                            set_groups(&mut pass, &flush_groups[index], image, clip, samplers);
                        }
                        pass.set_vertex_buffer(0, patch_vertices.slice(..));
                        pass.draw_path_patches(
                            draw.patch_index_range.clone(),
                            0,
                            draw.main_base_instance..draw.main_base_instance + draw.instance_count,
                        );
                        if let Some(buffer) = triangle_upload {
                            pass.set_pipeline(match (draw.kind, draw.has_clip_rect) {
                                (ClockwiseAtomicDrawKind::ClippedContent, true) => {
                                    &self.clipped_interior_clip_rect
                                }
                                (ClockwiseAtomicDrawKind::ClippedContent, false) => {
                                    &self.clipped_interior
                                }
                                (_, true) => &self.interior_clip_rect,
                                (_, false) => &self.interior,
                            });
                            if draw.kind == ClockwiseAtomicDrawKind::ClippedContent {
                                set_groups(
                                    &mut pass,
                                    &flush_groups[index],
                                    image,
                                    sampled_clip,
                                    samplers,
                                );
                            } else {
                                set_groups(&mut pass, &flush_groups[index], image, clip, samplers);
                            }
                            pass.set_vertex_buffer(0, buffer.slice());
                            for batch in draw.main_triangle_batches {
                                pass.draw(
                                    batch.vertex_start..batch.vertex_start + batch.vertex_count,
                                    0..batch.instance_count,
                                );
                            }
                        }
                    }
                    ClockwiseAtomicDrawKind::OutermostClip
                    | ClockwiseAtomicDrawKind::NestedClip => {
                        debug_assert_ne!(main_pass.kind, ClockwiseAtomicMainPassKind::Target);
                        let nested = draw.kind == ClockwiseAtomicDrawKind::NestedClip;
                        pass.set_pipeline(if nested {
                            &self.nested_clip_path
                        } else {
                            &self.outer_clip_path
                        });
                        set_groups(&mut pass, &flush_groups[index], image, clip, samplers);
                        pass.set_vertex_buffer(0, patch_vertices.slice(..));
                        pass.draw_path_patches(
                            draw.patch_index_range.clone(),
                            0,
                            draw.main_base_instance..draw.main_base_instance + draw.instance_count,
                        );
                        if let Some(buffer) = triangle_upload {
                            pass.set_pipeline(if nested {
                                &self.nested_clip_interior
                            } else {
                                &self.outer_clip_interior
                            });
                            set_groups(&mut pass, &flush_groups[index], image, clip, samplers);
                            pass.set_vertex_buffer(0, buffer.slice());
                            for batch in draw.main_triangle_batches {
                                pass.draw(
                                    batch.vertex_start..batch.vertex_start + batch.vertex_count,
                                    0..batch.instance_count,
                                );
                            }
                        }
                    }
                }
            }
            drop(pass);

            if capture_coverage
                && main_kinds[main_pass.draw_range.end - 1] == ClockwiseAtomicDrawKind::NestedClip
            {
                let clip_texture = clip_texture
                    .expect("nested CWA clip readback requires a clip render attachment");
                let readback = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("nuxie-cwa-nested-clip-readback"),
                    size: u64::from(clip_bytes_per_row) * u64::from(uniforms.render_target_height),
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
        debug_assert_eq!(next_triangle_upload, grouped_uploads.len());
        if let Some(readback) = &coverage_readback {
            coverage.copy_to_buffer(encoder, &readback.main);
        }
        coverage_readback.map(|mut readback| {
            readback.clip_updates = clip_update_readbacks;
            readback.clip_bytes_per_row = clip_bytes_per_row;
            readback.clip_height = uniforms.render_target_height;
            readback
        })
    }
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

fn cull_counterclockwise() -> wgpu::PrimitiveState {
    // C++ WebGPU declares CW front and culls Back. wgpu defaults to CCW front,
    // so culling Front preserves the same counterclockwise-face rejection.
    wgpu::PrimitiveState {
        cull_mode: Some(wgpu::Face::Front),
        ..Default::default()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_prefix_starts_nonzero_and_clears_only_at_generation_wrap() {
        let mut prefix = 0;
        assert!(rotate_coverage_prefix(&mut prefix));
        assert_eq!(prefix, CLOCKWISE_COVERAGE_PREFIX_STEP);

        assert!(!rotate_coverage_prefix(&mut prefix));
        assert_eq!(prefix, CLOCKWISE_COVERAGE_PREFIX_STEP * 2);

        prefix = !(CLOCKWISE_COVERAGE_PREFIX_STEP - 1);
        assert!(rotate_coverage_prefix(&mut prefix));
        assert_eq!(prefix, CLOCKWISE_COVERAGE_PREFIX_STEP);
    }

    #[test]
    fn retained_coverage_growth_uses_cpp_slack_without_exceeding_the_limit() {
        assert_eq!(retained_coverage_capacity_words(100, 1_000), 125);
        assert_eq!(retained_coverage_capacity_words(100, 110), 110);
        assert_eq!(retained_coverage_capacity_words(1, 1_000), 1);
        assert_eq!(retained_coverage_capacity_words(1_000, 1_000), 1_000);
    }

    #[test]
    fn coverage_capture_normalizes_current_generation_and_hides_stale_words() {
        let prefix = CLOCKWISE_COVERAGE_PREFIX_STEP * 2;
        assert_eq!(canonicalize_coverage_word(0, prefix), 0);
        assert_eq!(
            canonicalize_coverage_word(CLOCKWISE_COVERAGE_PREFIX_STEP | 0x3f800, prefix),
            0
        );
        assert_eq!(
            canonicalize_coverage_word(prefix | 0x40000, prefix),
            CLOCKWISE_COVERAGE_PREFIX_STEP | 0x40000
        );
    }

    #[test]
    fn retained_coverage_reuses_capacity_and_reclears_after_growth() {
        use crate::WgpuFactory;

        let factory = WgpuFactory::new(16, 16).unwrap();
        let pipeline = ClockwiseAtomicPipeline::new(&factory.context.device);
        let mut backing = pipeline.begin_frame_backing();

        let mut first_encoder = factory
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let first = backing.prepare_coverage(&factory.context.device, &mut first_encoder, 100);
        assert_eq!(first.prefix(), CLOCKWISE_COVERAGE_PREFIX_STEP);
        assert_eq!(backing.slot.coverage.as_ref().unwrap().capacity_words, 125);
        assert!(backing.clear_recorded_unsubmitted);
        factory.context.queue.submit(Some(first_encoder.finish()));
        factory
            .context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        backing.did_submit();

        let mut reuse_encoder = factory
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let reused = backing.prepare_coverage(&factory.context.device, &mut reuse_encoder, 125);
        assert_eq!(reused.prefix(), CLOCKWISE_COVERAGE_PREFIX_STEP * 2);
        assert_eq!(backing.slot.coverage.as_ref().unwrap().capacity_words, 125);
        assert!(!backing.clear_recorded_unsubmitted);

        let mut growth_encoder = factory
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let grown = backing.prepare_coverage(&factory.context.device, &mut growth_encoder, 126);
        assert_eq!(grown.prefix(), CLOCKWISE_COVERAGE_PREFIX_STEP);
        assert_eq!(backing.slot.coverage.as_ref().unwrap().capacity_words, 157);
        assert!(backing.clear_recorded_unsubmitted);
    }

    #[test]
    fn abandoned_coverage_clear_remains_required_when_its_slot_returns() {
        use crate::WgpuFactory;

        let factory = WgpuFactory::new(16, 16).unwrap();
        let pipeline = ClockwiseAtomicPipeline::new(&factory.context.device);
        {
            let mut backing = pipeline.begin_frame_backing();
            let mut encoder = factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            let first = backing.prepare_coverage(&factory.context.device, &mut encoder, 1);
            assert_eq!(first.prefix(), CLOCKWISE_COVERAGE_PREFIX_STEP);
            assert!(backing.clear_recorded_unsubmitted);
            drop(encoder);
        }
        // Walk the other two ring slots so the abandoned slot is acquired again.
        drop(pipeline.begin_frame_backing());
        drop(pipeline.begin_frame_backing());

        let mut backing = pipeline.begin_frame_backing();
        let mut encoder = factory
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let retry = backing.prepare_coverage(&factory.context.device, &mut encoder, 1);
        assert_eq!(retry.prefix(), CLOCKWISE_COVERAGE_PREFIX_STEP);
        assert!(backing.clear_recorded_unsubmitted);
    }

    #[test]
    fn main_passes_stop_at_clip_visibility_clear_and_readback_boundaries() {
        use ClockwiseAtomicDrawKind::{
            ClippedContent as Clipped, Content, NestedClip as Nested, OutermostClip as Outermost,
        };
        use ClockwiseAtomicMainPassKind::{ClipClear, ClipLoad, Target};

        let kinds = [
            Content, Clipped, Outermost, Nested, Nested, Content, Nested, Outermost, Nested,
        ];
        let pass = |kind, draw_range| ClockwiseAtomicMainPass { kind, draw_range };

        assert_eq!(
            clockwise_atomic_main_passes(&kinds, false),
            [
                pass(Target, 0..2),
                pass(ClipClear, 2..5),
                pass(Target, 5..6),
                pass(ClipLoad, 6..7),
                pass(ClipClear, 7..9),
            ]
        );
        assert_eq!(
            clockwise_atomic_main_passes(&kinds, true),
            [
                pass(Target, 0..2),
                pass(ClipClear, 2..4),
                pass(ClipLoad, 4..5),
                pass(Target, 5..6),
                pass(ClipLoad, 6..7),
                pass(ClipClear, 7..9),
            ]
        );
        assert!(clockwise_atomic_main_passes(&[], true).is_empty());
    }

    #[cfg(feature = "perf-counters")]
    #[test]
    fn additional_complex_content_reuses_clockwise_atomic_passes() {
        use crate::{RenderMode, WgpuFactory, WgpuPaint, WgpuPath};
        use nuxie_render_api::{FillRule, RawPath, Renderer};
        use std::sync::Arc;

        let compound = |left: f32, right: f32| {
            let mut raw_path = RawPath::new();
            for [top, bottom] in [[10.0, 245.0], [265.0, 500.0]] {
                raw_path.move_to(left, top);
                raw_path.line_to(right, top);
                raw_path.line_to(right, bottom);
                raw_path.line_to(left, bottom);
                raw_path.close();
            }
            WgpuPath {
                raw_path: Arc::new(raw_path),
                fill_rule: FillRule::NonZero,
                valid: true,
            }
        };
        let left = compound(10.0, 245.0);
        let right = compound(265.0, 500.0);
        let factory = WgpuFactory::new_with_mode(512, 512, RenderMode::ClockwiseAtomic).unwrap();
        let paint = WgpuPaint::default();
        let work = |paths: &[&WgpuPath]| {
            let mut frame = factory.begin_frame_for_benchmark(0xffff_ffff, true);
            for path in paths {
                frame.draw_path(*path, &paint);
            }
            frame.finish_for_benchmark().unwrap().backend_work
        };

        let one_draw = work(&[&left]);
        let two_draws = work(&[&left, &right]);
        let third_slot = work(&[&left]);
        let reused_slot = work(&[&left]);
        // The second path needs one more tessellation pass, but reuses the
        // flush-wide borrowed and main CWA passes. Coverage is retained on its
        // own frame slot. Upload counters report the actual aligned staging
        // copy commands rather than aggregate destination-page occupancy.
        assert_eq!((one_draw.render_passes, two_draws.render_passes), (4, 5));
        assert_eq!(
            [
                one_draw.buffer_upload_calls,
                two_draws.buffer_upload_calls,
                third_slot.buffer_upload_calls,
                reused_slot.buffer_upload_calls,
            ],
            [5, 6, 5, 5]
        );
        assert_eq!(
            [
                one_draw.buffer_upload_bytes,
                two_draws.buffer_upload_bytes,
                third_slot.buffer_upload_bytes,
                reused_slot.buffer_upload_bytes,
            ],
            [1_504, 2_368, 1_504, 1_504]
        );
        // The first traversal allocates and clears all three ring slots. The
        // fourth frame reuses slot zero with a new prefix and no physical clear.
        assert_eq!(
            [
                one_draw.buffer_clear_calls,
                two_draws.buffer_clear_calls,
                third_slot.buffer_clear_calls,
                reused_slot.buffer_clear_calls,
            ],
            [1, 1, 1, 0]
        );
        assert_eq!(
            [
                one_draw.buffer_clear_bytes,
                two_draws.buffer_clear_bytes,
                third_slot.buffer_clear_bytes,
                reused_slot.buffer_clear_bytes,
            ],
            [655_360, 1_310_720, 655_360, 0]
        );
        assert_eq!(
            [
                one_draw.command_encoders,
                two_draws.command_encoders,
                third_slot.command_encoders,
                reused_slot.command_encoders,
            ],
            [1; 4]
        );
        assert_eq!(
            [
                one_draw.queue_submissions,
                two_draws.queue_submissions,
                third_slot.queue_submissions,
                reused_slot.queue_submissions,
            ],
            [1; 4]
        );
        assert!(two_draws.gpu_draw_calls > one_draw.gpu_draw_calls);
    }
}
