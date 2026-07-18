//! GPU tessellation pass translated from `renderer/src/shaders/tessellate.glsl`.

use crate::gpu::{ContourData, FlushUniforms, PaintAuxData, PaintData, PathData, TessVertexSpan};
use crate::work_metrics::{record_buffer_upload, CountedCommandEncoderExt, CountedDeviceExt};
use bytemuck::Zeroable;
#[cfg(feature = "perf-diagnostics")]
use std::time::Instant;
use std::{
    num::NonZeroU64,
    ops::Deref,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, MutexGuard,
    },
};

// Mirrors C++ `gpu::kBufferRingSize`; the frame guard owns its slot through GPU completion.
const BUFFER_RING_SIZE: usize = 3;
const MIN_UPLOAD_CAPACITY: u64 = 4 * 1024;
const STAGING_CHUNK_SIZE: u64 = 64 * 1024;
const MAX_CACHED_TESSELLATION_TEXTURES: usize = 1;
// C++ keeps its first 1 MiB transient arena block but releases overflow blocks
// after a flush. The typed Rust scratch grows naturally and uses this only as
// an aggregate retention ceiling; it does not eagerly reserve the arena size.
const MAX_RETAINED_MSAA_PACKING_SCRATCH_BYTES: usize = 1024 * 1024;

#[derive(Default)]
pub(crate) struct MsaaPackingScratch {
    pub(crate) spans: Vec<TessVertexSpan>,
    pub(crate) contours: Vec<ContourData>,
    pub(crate) local_contour_ids: Vec<u32>,
    pub(crate) paths: Vec<PathData>,
    pub(crate) paints: Vec<PaintData>,
    pub(crate) paint_aux: Vec<PaintAuxData>,
    #[cfg(feature = "perf-diagnostics")]
    diagnostics: MsaaPackingScratchDiagnostics,
    #[cfg(feature = "perf-diagnostics")]
    capacity_before_flush_bytes: usize,
}

#[cfg(feature = "perf-diagnostics")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct MsaaPackingScratchDiagnostics {
    flushes: u64,
    capacity_growths: u64,
    peak_capacity_bytes: u64,
    released_capacity_bytes: u64,
}

impl MsaaPackingScratch {
    pub(crate) fn begin_logical_flush(&mut self, path_count: usize) {
        #[cfg(feature = "perf-diagnostics")]
        {
            self.capacity_before_flush_bytes = self.retained_capacity_bytes();
        }
        self.clear();

        let entries_with_sentinel = path_count
            .checked_add(1)
            .expect("MSAA logical-flush path count overflow");
        self.paths.reserve(entries_with_sentinel);
        self.paints.reserve(entries_with_sentinel);
        self.paint_aux.reserve(entries_with_sentinel);
        self.paths.push(PathData::zeroed());
        self.paints.push(PaintData::zeroed());
        self.paint_aux.push(PaintAuxData::zeroed());
    }

    #[cfg(feature = "perf-diagnostics")]
    pub(crate) fn finish_logical_flush(&mut self) {
        let retained_capacity_bytes = self.retained_capacity_bytes();
        self.diagnostics.flushes = self.diagnostics.flushes.saturating_add(1);
        if retained_capacity_bytes > self.capacity_before_flush_bytes {
            self.diagnostics.capacity_growths = self.diagnostics.capacity_growths.saturating_add(1);
        }
        self.diagnostics.peak_capacity_bytes = self
            .diagnostics
            .peak_capacity_bytes
            .max(u64::try_from(retained_capacity_bytes).unwrap_or(u64::MAX));
    }

    fn clear(&mut self) {
        self.spans.clear();
        self.contours.clear();
        self.local_contour_ids.clear();
        self.paths.clear();
        self.paints.clear();
        self.paint_aux.clear();
    }

    fn finish_submission(&mut self) {
        self.clear();
        let retained_before = self.retained_capacity_bytes();
        if retained_before > MAX_RETAINED_MSAA_PACKING_SCRATCH_BYTES {
            self.spans = Vec::new();
            self.contours = Vec::new();
            self.local_contour_ids = Vec::new();
            self.paths = Vec::new();
            self.paints = Vec::new();
            self.paint_aux = Vec::new();
        }
        #[cfg(feature = "perf-diagnostics")]
        {
            let retained_after = self.retained_capacity_bytes();
            self.diagnostics.released_capacity_bytes =
                self.diagnostics.released_capacity_bytes.saturating_add(
                    u64::try_from(retained_before.saturating_sub(retained_after))
                        .unwrap_or(u64::MAX),
                );
        }
    }

    fn retained_capacity_bytes(&self) -> usize {
        self.capacities()
            .into_iter()
            .zip([
                size_of::<TessVertexSpan>(),
                size_of::<ContourData>(),
                size_of::<u32>(),
                size_of::<PathData>(),
                size_of::<PaintData>(),
                size_of::<PaintAuxData>(),
            ])
            .fold(0usize, |bytes, (capacity, element_size)| {
                bytes.saturating_add(capacity.saturating_mul(element_size))
            })
    }

    fn capacities(&self) -> [usize; 6] {
        [
            self.spans.capacity(),
            self.contours.capacity(),
            self.local_contour_ids.capacity(),
            self.paths.capacity(),
            self.paints.capacity(),
            self.paint_aux.capacity(),
        ]
    }
}

#[cfg(feature = "perf-diagnostics")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TessellationUploadDiagnostics {
    pub submissions: u64,
    pub upload_calls: u64,
    pub populated_pages: u64,
    pub page_allocations: u64,
    pub payload_bytes: u64,
    pub used_bytes: u64,
    pub written_bytes: u64,
    pub populated_capacity_bytes: u64,
    pub cpu_pack_ns: u64,
    pub write_buffer_ns: u64,
    pub msaa_packing_flushes: u64,
    pub msaa_packing_capacity_growths: u64,
    pub msaa_packing_peak_capacity_bytes: u64,
    pub msaa_packing_retained_capacity_bytes: u64,
    pub msaa_packing_released_capacity_bytes: u64,
}

pub(crate) struct Tessellator {
    pub pipeline: wgpu::RenderPipeline,
    pub flush_layout: wgpu::BindGroupLayout,
    pub span_indices: wgpu::Buffer,
    _linear_sampler: wgpu::Sampler,
    sampler_group: wgpu::BindGroup,
    upload_slots: [Mutex<TessellationUploadSlot>; BUFFER_RING_SIZE],
    next_upload_slot: AtomicUsize,
    texture_pool: TessellationTexturePool,
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
        let span_indices = device.create_counted_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("nuxie-tessellation-span-indices"),
            contents: bytemuck::cast_slice(&[0u16, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7]),
            usage: wgpu::BufferUsages::INDEX,
        });
        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nuxie-tessellation-linear-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let sampler_group = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-tessellation-sampler-group"),
            layout: &sampler_layout,
            entries: &[binding(10, wgpu::BindingResource::Sampler(&linear_sampler))],
        });
        let limits = device.limits();
        Self {
            pipeline,
            flush_layout,
            span_indices,
            _linear_sampler: linear_sampler,
            sampler_group,
            upload_slots: std::array::from_fn(|_| {
                Mutex::new(TessellationUploadSlot::new(device, &limits))
            }),
            next_upload_slot: AtomicUsize::new(0),
            texture_pool: TessellationTexturePool::default(),
        }
    }

    pub(crate) fn begin_frame_uploads(&self, device: &wgpu::Device) -> TessellationUploadFrame<'_> {
        let slot_index = self.next_upload_slot.fetch_add(1, Ordering::Relaxed) % BUFFER_RING_SIZE;
        let mut slot = self.upload_slots[slot_index]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        #[cfg(feature = "perf-diagnostics")]
        slot.reset_diagnostics();
        slot.begin_submission(device);
        TessellationUploadFrame { slot }
    }

    pub(crate) fn begin_frame_textures(&self) -> TessellationTextureFrame<'_> {
        TessellationTextureFrame {
            pool: &self.texture_pool,
            checked_out: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn cached_texture(&self) -> Option<Arc<TessellationTexture>> {
        self.texture_pool.cached()
    }

    #[cfg(test)]
    pub(crate) fn cached_texture_len(&self) -> usize {
        self.texture_pool.cached_len()
    }

    pub(crate) fn encode(
        &self,
        device: &wgpu::Device,
        textures: &mut TessellationTextureFrame<'_>,
        uploads: &mut TessellationUploadFrame<'_>,
        encoder: &mut wgpu::CommandEncoder,
        feather_lut: &wgpu::TextureView,
        spans: &[TessVertexSpan],
        uniforms: &FlushUniforms,
        paths: &[PathData],
        contours: &[ContourData],
        height: u32,
    ) -> Arc<TessellationTexture> {
        self.encode_with_new_flush_resources(
            device,
            textures,
            uploads,
            encoder,
            feather_lut,
            spans,
            uniforms,
            paths,
            contours,
            height,
        )
        .texture
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_with_new_flush_resources(
        &self,
        device: &wgpu::Device,
        textures: &mut TessellationTextureFrame<'_>,
        uploads: &mut TessellationUploadFrame<'_>,
        encoder: &mut wgpu::CommandEncoder,
        feather_lut: &wgpu::TextureView,
        spans: &[TessVertexSpan],
        uniforms: &FlushUniforms,
        paths: &[PathData],
        contours: &[ContourData],
        height: u32,
    ) -> TessellationEncoding {
        assert!(!spans.is_empty() && !paths.is_empty());
        let span_buffer = uploads.upload_spans(device, encoder, bytemuck::cast_slice(spans));
        let flush_resources =
            uploads.upload_flush_resources(device, encoder, uniforms, paths, contours);
        let texture = self.encode_uploaded(
            device,
            textures,
            encoder,
            feather_lut,
            &span_buffer,
            &flush_resources,
            spans.len(),
            height,
        );
        TessellationEncoding {
            texture,
            flush_resources,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_with_new_flush_resources_and_prefixes(
        &self,
        device: &wgpu::Device,
        textures: &mut TessellationTextureFrame<'_>,
        uploads: &mut TessellationUploadFrame<'_>,
        encoder: &mut wgpu::CommandEncoder,
        feather_lut: &wgpu::TextureView,
        prefixes: [&[u8]; 2],
        spans: &[TessVertexSpan],
        uniforms: &FlushUniforms,
        paths: &[PathData],
        contours: &[ContourData],
        height: u32,
    ) -> TessellationEncodingWithPrefixes {
        assert!(prefixes.iter().all(|prefix| !prefix.is_empty()));
        assert!(!spans.is_empty() && !paths.is_empty());
        let dummy_contours = [ContourData::zeroed()];
        let contours = if contours.is_empty() {
            &dummy_contours[..]
        } else {
            contours
        };
        let storage_alignment = uploads.slot.storage_alignment;
        let uniform_alignment = uploads.slot.uniform_alignment;
        let [prefix_0, prefix_1, span_buffer, uniform_buffer, path_buffer, contour_buffer] =
            uploads.slot.uploads.upload_group(
                device,
                encoder,
                [
                    UploadRequest::new(prefixes[0], storage_alignment),
                    UploadRequest::new(prefixes[1], storage_alignment),
                    UploadRequest::new(bytemuck::cast_slice(spans), wgpu::COPY_BUFFER_ALIGNMENT),
                    UploadRequest::new(bytemuck::bytes_of(uniforms), uniform_alignment),
                    UploadRequest::new(bytemuck::cast_slice(paths), storage_alignment),
                    UploadRequest::new(bytemuck::cast_slice(contours), storage_alignment),
                ],
            );
        let flush_resources = TessellationFlushResources {
            uniform_buffer,
            path_buffer,
            contour_buffer,
        };
        let texture = self.encode_uploaded(
            device,
            textures,
            encoder,
            feather_lut,
            &span_buffer,
            &flush_resources,
            spans.len(),
            height,
        );
        TessellationEncodingWithPrefixes {
            tessellation: TessellationEncoding {
                texture,
                flush_resources,
            },
            prefixes: [prefix_0, prefix_1],
        }
    }

    pub(crate) fn encode_with_flush_resources(
        &self,
        device: &wgpu::Device,
        textures: &mut TessellationTextureFrame<'_>,
        uploads: &mut TessellationUploadFrame<'_>,
        encoder: &mut wgpu::CommandEncoder,
        feather_lut: &wgpu::TextureView,
        spans: &[TessVertexSpan],
        flush_resources: &TessellationFlushResources,
        height: u32,
    ) -> Arc<TessellationTexture> {
        assert!(!spans.is_empty());
        let span_buffer = uploads.upload_spans(device, encoder, bytemuck::cast_slice(spans));
        self.encode_uploaded(
            device,
            textures,
            encoder,
            feather_lut,
            &span_buffer,
            flush_resources,
            spans.len(),
            height,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_uploaded(
        &self,
        device: &wgpu::Device,
        textures: &mut TessellationTextureFrame<'_>,
        encoder: &mut wgpu::CommandEncoder,
        feather_lut: &wgpu::TextureView,
        span_buffer: &UploadSlice,
        flush_resources: &TessellationFlushResources,
        span_count: usize,
        height: u32,
    ) -> Arc<TessellationTexture> {
        let flush_group = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-tessellation-flush-group"),
            layout: &self.flush_layout,
            entries: &[
                binding(0, flush_resources.uniform_binding()),
                binding(3, flush_resources.path_binding()),
                binding(6, flush_resources.contour_binding()),
                binding(10, wgpu::BindingResource::TextureView(feather_lut)),
            ],
        });
        let texture = textures.checkout(device, height);
        let mut pass = encoder.begin_counted_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("nuxie-tessellation-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture.view,
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
        pass.set_bind_group(3, &self.sampler_group, &[]);
        pass.set_vertex_buffer(0, span_buffer.slice());
        pass.set_index_buffer(self.span_indices.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_tessellation_spans(0..12, 0, 0..span_count as u32);
        drop(pass);
        texture
    }
}

pub(crate) struct TessellationTexture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) height: u32,
}

impl TessellationTexture {
    fn new(device: &wgpu::Device, height: u32) -> Self {
        let height = height.max(1);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-tessellation-data"),
            size: wgpu::Extent3d {
                width: 2048,
                height,
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
        Self {
            texture,
            view,
            height,
        }
    }
}

impl Deref for TessellationTexture {
    type Target = wgpu::Texture;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

#[derive(Default)]
struct TessellationTexturePool {
    available: Mutex<Vec<Arc<TessellationTexture>>>,
}

impl TessellationTexturePool {
    fn checkout(&self, device: &wgpu::Device, height: u32) -> Arc<TessellationTexture> {
        let height = height.max(1);
        let mut available = self
            .available
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(index) = available
            .iter()
            .position(|texture| texture.height == height)
        {
            return available.swap_remove(index);
        }
        available.clear();
        Arc::new(TessellationTexture::new(device, height))
    }

    fn recycle(&self, texture: Arc<TessellationTexture>) {
        let mut available = self
            .available
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if available.len() < MAX_CACHED_TESSELLATION_TEXTURES {
            available.push(texture);
        }
    }

    #[cfg(test)]
    fn cached(&self) -> Option<Arc<TessellationTexture>> {
        self.available
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .last()
            .cloned()
    }

    #[cfg(test)]
    fn cached_len(&self) -> usize {
        self.available
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}

pub(crate) struct TessellationTextureFrame<'a> {
    pool: &'a TessellationTexturePool,
    checked_out: Vec<Arc<TessellationTexture>>,
}

impl TessellationTextureFrame<'_> {
    pub(crate) fn checkout(
        &mut self,
        device: &wgpu::Device,
        height: u32,
    ) -> Arc<TessellationTexture> {
        let texture = self.pool.checkout(device, height);
        self.checked_out.push(Arc::clone(&texture));
        texture
    }

    pub(crate) fn recycle(&mut self) {
        if let Some(texture) = self
            .checked_out
            .drain(..)
            .max_by_key(|texture| texture.height)
        {
            self.pool.recycle(texture);
        }
    }
}

pub(crate) struct TessellationUploadFrame<'a> {
    slot: MutexGuard<'a, TessellationUploadSlot>,
}

impl Drop for TessellationUploadFrame<'_> {
    fn drop(&mut self) {
        // The frame already owns the slot guard. Logical-flush scratch guards
        // are scoped inside packing, so every path keeps the slot -> scratch
        // lock order, including unwind and recoverable-error cleanup.
        self.slot.discard_active_uploads();
        self.slot.finish_msaa_packing_scratch();
    }
}

#[derive(Clone, Copy)]
pub(crate) enum FrameUploadPayload<'a> {
    Storage(&'a [u8]),
    Vertex(&'a [u8]),
}

impl FrameUploadPayload<'_> {
    fn size(self) -> u64 {
        match self {
            Self::Storage(bytes) | Self::Vertex(bytes) => bytes.len() as u64,
        }
    }

    fn alignment(self, storage_alignment: u64) -> u64 {
        match self {
            Self::Storage(_) => storage_alignment,
            Self::Vertex(_) => wgpu::COPY_BUFFER_ALIGNMENT,
        }
    }
}

impl TessellationUploadFrame<'_> {
    pub(crate) fn msaa_packing_scratch(&self) -> Arc<Mutex<MsaaPackingScratch>> {
        Arc::clone(&self.slot.msaa_packing_scratch)
    }

    fn upload_spans(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        bytes: &[u8],
    ) -> UploadSlice {
        self.slot
            .uploads
            .upload(device, encoder, bytes, wgpu::COPY_BUFFER_ALIGNMENT)
    }

    pub(crate) fn upload_uniforms(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        bytes: &[u8],
    ) -> UploadSlice {
        let alignment = self.slot.uniform_alignment;
        self.slot.uploads.upload(device, encoder, bytes, alignment)
    }

    pub(crate) fn upload_group(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        payloads: &[FrameUploadPayload<'_>],
    ) -> Vec<UploadSlice> {
        let storage_alignment = self.slot.storage_alignment;
        self.slot
            .uploads
            .upload_frame_group(device, encoder, payloads, storage_alignment)
    }

    fn upload_paths(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        bytes: &[u8],
    ) -> UploadSlice {
        let alignment = self.slot.storage_alignment;
        self.slot.uploads.upload(device, encoder, bytes, alignment)
    }

    fn upload_contours(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        bytes: &[u8],
    ) -> UploadSlice {
        let alignment = self.slot.storage_alignment;
        self.slot.uploads.upload(device, encoder, bytes, alignment)
    }

    pub(crate) fn upload_flush_resources(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        uniforms: &FlushUniforms,
        paths: &[PathData],
        contours: &[ContourData],
    ) -> TessellationFlushResources {
        assert!(!paths.is_empty());
        let dummy_contours = [ContourData::zeroed()];
        TessellationFlushResources {
            uniform_buffer: self.upload_uniforms(device, encoder, bytemuck::bytes_of(uniforms)),
            path_buffer: self.upload_paths(device, encoder, bytemuck::cast_slice(paths)),
            contour_buffer: self.upload_contours(
                device,
                encoder,
                bytemuck::cast_slice(if contours.is_empty() {
                    &dummy_contours
                } else {
                    contours
                }),
            ),
        }
    }

    pub(crate) fn finish_submission(&mut self, encoder: &wgpu::CommandEncoder) {
        self.slot.finish_submission(encoder);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn begin_next_submission(&mut self, device: &wgpu::Device) {
        self.slot.begin_submission(device);
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn begin_next_submission_without_reuse(&mut self) {
        self.slot.uploads.pages.clear();
    }

    #[cfg(feature = "perf-diagnostics")]
    pub(crate) fn diagnostics(&self) -> TessellationUploadDiagnostics {
        let mut diagnostics = self.slot.uploads.diagnostics;
        let scratch = self
            .slot
            .msaa_packing_scratch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        diagnostics.msaa_packing_flushes = scratch.diagnostics.flushes;
        diagnostics.msaa_packing_capacity_growths = scratch.diagnostics.capacity_growths;
        diagnostics.msaa_packing_peak_capacity_bytes = scratch.diagnostics.peak_capacity_bytes;
        diagnostics.msaa_packing_retained_capacity_bytes =
            u64::try_from(scratch.retained_capacity_bytes()).unwrap_or(u64::MAX);
        diagnostics.msaa_packing_released_capacity_bytes =
            scratch.diagnostics.released_capacity_bytes;
        diagnostics
    }
}

struct TessellationUploadSlot {
    uploads: UploadArena,
    msaa_packing_scratch: Arc<Mutex<MsaaPackingScratch>>,
    uniform_alignment: u64,
    storage_alignment: u64,
}

impl TessellationUploadSlot {
    fn new(device: &wgpu::Device, limits: &wgpu::Limits) -> Self {
        Self {
            uploads: UploadArena::new(
                device,
                "nuxie-tessellation-upload-ring",
                wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::STORAGE,
            ),
            msaa_packing_scratch: Arc::new(Mutex::new(MsaaPackingScratch::default())),
            uniform_alignment: limits.min_uniform_buffer_offset_alignment as u64,
            storage_alignment: limits.min_storage_buffer_offset_alignment as u64,
        }
    }

    fn begin_submission(&mut self, device: &wgpu::Device) {
        self.uploads.begin_submission(device);
    }

    #[cfg(feature = "perf-diagnostics")]
    fn reset_diagnostics(&mut self) {
        self.uploads.diagnostics = TessellationUploadDiagnostics::default();
        let mut scratch = self
            .msaa_packing_scratch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        scratch.diagnostics = MsaaPackingScratchDiagnostics::default();
        scratch.capacity_before_flush_bytes = scratch.retained_capacity_bytes();
    }

    fn finish_submission(&mut self, encoder: &wgpu::CommandEncoder) {
        self.uploads.finish_submission(encoder);
        self.finish_msaa_packing_scratch();
    }

    fn finish_msaa_packing_scratch(&self) {
        self.msaa_packing_scratch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .finish_submission();
    }

    fn discard_active_uploads(&mut self) {
        self.uploads.discard_active_writes();
    }
}

struct UploadArena {
    label: &'static str,
    usage: wgpu::BufferUsages,
    pages: Vec<UploadPage>,
    device: wgpu::Device,
    staging_belt: wgpu::util::StagingBelt,
    has_active_staging_writes: bool,
    #[cfg(test)]
    abandoned_staging_belt_resets: u64,
    #[cfg(feature = "perf-diagnostics")]
    diagnostics: TessellationUploadDiagnostics,
}

#[derive(Clone, Copy)]
struct UploadRequest<'a> {
    bytes: &'a [u8],
    alignment: u64,
}

impl<'a> UploadRequest<'a> {
    fn new(bytes: &'a [u8], alignment: u64) -> Self {
        Self { bytes, alignment }
    }
}

impl UploadArena {
    fn new(device: &wgpu::Device, label: &'static str, usage: wgpu::BufferUsages) -> Self {
        Self {
            label,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            pages: Vec::new(),
            device: device.clone(),
            staging_belt: wgpu::util::StagingBelt::new(device.clone(), STAGING_CHUNK_SIZE),
            has_active_staging_writes: false,
            #[cfg(test)]
            abandoned_staging_belt_resets: 0,
            #[cfg(feature = "perf-diagnostics")]
            diagnostics: TessellationUploadDiagnostics::default(),
        }
    }

    fn begin_submission(&mut self, device: &wgpu::Device) {
        if self.pages.len() > 1 {
            let previous_usage = self.pages.iter().map(|page| page.used).sum::<u64>();
            self.pages.clear();
            self.pages.push(UploadPage::new(
                device,
                self.label,
                self.usage,
                upload_capacity(previous_usage),
            ));
            #[cfg(feature = "perf-diagnostics")]
            {
                self.diagnostics.page_allocations =
                    self.diagnostics.page_allocations.saturating_add(1);
            }
        }
        for page in &mut self.pages {
            page.used = 0;
        }
    }

    fn upload(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        bytes: &[u8],
        alignment: u64,
    ) -> UploadSlice {
        #[cfg(feature = "perf-diagnostics")]
        let started = Instant::now();
        assert!(!bytes.is_empty());
        let size = bytes.len() as u64;
        let write_size = align_u64(size, wgpu::COPY_BUFFER_ALIGNMENT);
        let alignment = alignment.max(wgpu::COPY_BUFFER_ALIGNMENT);
        #[cfg(feature = "perf-diagnostics")]
        let page_count_before = self.pages.len();
        let page_index = self
            .pages
            .last()
            .and_then(|page| {
                let offset = align_u64(page.used, alignment);
                (offset.saturating_add(write_size) <= page.capacity).then_some(self.pages.len() - 1)
            })
            .unwrap_or_else(|| {
                let previous_capacity = self.pages.last().map_or(0, |page| page.capacity);
                self.pages.push(UploadPage::new(
                    device,
                    self.label,
                    self.usage,
                    upload_capacity(write_size.max(previous_capacity)),
                ));
                self.pages.len() - 1
            });
        let (upload, destination) = {
            let page = &mut self.pages[page_index];
            let offset = align_u64(page.used, alignment);
            let end = offset
                .checked_add(write_size)
                .expect("tessellation upload overflow");
            page.used = end;
            let buffer = page.buffer.clone();
            (
                UploadSlice {
                    buffer: buffer.clone(),
                    offset,
                    size: NonZeroU64::new(size).expect("nonempty tessellation upload"),
                },
                buffer,
            )
        };
        #[cfg(feature = "perf-diagnostics")]
        let write_started = Instant::now();
        // Record the copy at allocation time so it precedes the first render
        // pass that can consume the returned slice. `finish_submission` only
        // closes and schedules recall of the mapped staging chunks; deferring
        // copies there would place them after their consumers.
        self.has_active_staging_writes = true;
        let mut mapped = self.staging_belt.write_buffer(
            encoder,
            &destination,
            upload.offset,
            NonZeroU64::new(write_size).expect("nonempty aligned tessellation upload"),
        );
        record_buffer_upload(write_size);
        mapped.slice(..bytes.len()).copy_from_slice(bytes);
        mapped.slice(bytes.len()..).fill(0);
        drop(mapped);
        #[cfg(feature = "perf-diagnostics")]
        {
            self.diagnostics.upload_calls = self.diagnostics.upload_calls.saturating_add(1);
            self.diagnostics.written_bytes =
                self.diagnostics.written_bytes.saturating_add(write_size);
            self.diagnostics.payload_bytes = self.diagnostics.payload_bytes.saturating_add(size);
            self.diagnostics.page_allocations = self
                .diagnostics
                .page_allocations
                .saturating_add((self.pages.len() - page_count_before) as u64);
            self.diagnostics.cpu_pack_ns = self
                .diagnostics
                .cpu_pack_ns
                .saturating_add(elapsed_ns(started));
            self.diagnostics.write_buffer_ns = self
                .diagnostics
                .write_buffer_ns
                .saturating_add(elapsed_ns(write_started));
        }
        upload
    }

    fn upload_frame_group(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        payloads: &[FrameUploadPayload<'_>],
        storage_alignment: u64,
    ) -> Vec<UploadSlice> {
        #[cfg(feature = "perf-diagnostics")]
        let started = Instant::now();
        let (relative_offsets, write_size, group_alignment) =
            frame_upload_group_layout(payloads, storage_alignment);
        #[cfg(feature = "perf-diagnostics")]
        let page_count_before = self.pages.len();
        let page_index = self
            .pages
            .last()
            .and_then(|page| {
                let offset = align_u64(page.used, group_alignment);
                (offset.saturating_add(write_size) <= page.capacity).then_some(self.pages.len() - 1)
            })
            .unwrap_or_else(|| {
                let previous_capacity = self.pages.last().map_or(0, |page| page.capacity);
                self.pages.push(UploadPage::new(
                    device,
                    self.label,
                    self.usage,
                    upload_capacity(write_size.max(previous_capacity)),
                ));
                self.pages.len() - 1
            });
        let (base_offset, destination) = {
            let page = &mut self.pages[page_index];
            let base_offset = align_u64(page.used, group_alignment);
            page.used = base_offset
                .checked_add(write_size)
                .expect("grouped frame upload overflow");
            (base_offset, page.buffer.clone())
        };
        let uploads = payloads
            .iter()
            .zip(&relative_offsets)
            .map(|(payload, relative_offset)| UploadSlice {
                buffer: destination.clone(),
                offset: base_offset + relative_offset,
                size: NonZeroU64::new(payload.size()).expect("nonempty grouped frame upload"),
            })
            .collect::<Vec<_>>();
        #[cfg(feature = "perf-diagnostics")]
        let write_started = Instant::now();
        self.has_active_staging_writes = true;
        let mut mapped = self.staging_belt.write_buffer(
            encoder,
            &destination,
            base_offset,
            NonZeroU64::new(write_size).expect("nonempty grouped frame upload"),
        );
        record_buffer_upload(write_size);
        mapped.slice(..).fill(0);
        for (payload, offset) in payloads.iter().zip(relative_offsets) {
            let (FrameUploadPayload::Storage(bytes) | FrameUploadPayload::Vertex(bytes)) = payload;
            let start = usize::try_from(offset).expect("grouped upload offset fits usize");
            let end = start
                .checked_add(bytes.len())
                .expect("grouped upload range overflow");
            mapped.slice(start..end).copy_from_slice(bytes);
        }
        drop(mapped);
        #[cfg(feature = "perf-diagnostics")]
        {
            self.diagnostics.upload_calls = self.diagnostics.upload_calls.saturating_add(1);
            self.diagnostics.written_bytes =
                self.diagnostics.written_bytes.saturating_add(write_size);
            self.diagnostics.payload_bytes = self
                .diagnostics
                .payload_bytes
                .saturating_add(payloads.iter().map(|payload| payload.size()).sum::<u64>());
            self.diagnostics.page_allocations = self
                .diagnostics
                .page_allocations
                .saturating_add((self.pages.len() - page_count_before) as u64);
            self.diagnostics.cpu_pack_ns = self
                .diagnostics
                .cpu_pack_ns
                .saturating_add(elapsed_ns(started));
            self.diagnostics.write_buffer_ns = self
                .diagnostics
                .write_buffer_ns
                .saturating_add(elapsed_ns(write_started));
        }
        uploads
    }

    fn upload_group<const N: usize>(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        requests: [UploadRequest<'_>; N],
    ) -> [UploadSlice; N] {
        #[cfg(feature = "perf-diagnostics")]
        let started = Instant::now();
        assert!(N != 0 && requests.iter().all(|request| !request.bytes.is_empty()));
        let sizes = requests.map(|request| request.bytes.len() as u64);
        let alignments = requests.map(|request| request.alignment);
        let (relative_offsets, write_size) = upload_group_layout(sizes, alignments);
        let group_alignment = alignments
            .into_iter()
            .max()
            .unwrap_or(wgpu::COPY_BUFFER_ALIGNMENT)
            .max(wgpu::COPY_BUFFER_ALIGNMENT);
        #[cfg(feature = "perf-diagnostics")]
        let page_count_before = self.pages.len();
        let page_index = self
            .pages
            .last()
            .and_then(|page| {
                let offset = align_u64(page.used, group_alignment);
                (offset.saturating_add(write_size) <= page.capacity).then_some(self.pages.len() - 1)
            })
            .unwrap_or_else(|| {
                let previous_capacity = self.pages.last().map_or(0, |page| page.capacity);
                self.pages.push(UploadPage::new(
                    device,
                    self.label,
                    self.usage,
                    upload_capacity(write_size.max(previous_capacity)),
                ));
                self.pages.len() - 1
            });
        let (base_offset, destination) = {
            let page = &mut self.pages[page_index];
            let base_offset = align_u64(page.used, group_alignment);
            page.used = base_offset
                .checked_add(write_size)
                .expect("grouped tessellation upload overflow");
            (base_offset, page.buffer.clone())
        };
        let uploads = std::array::from_fn(|index| UploadSlice {
            buffer: destination.clone(),
            offset: base_offset + relative_offsets[index],
            size: NonZeroU64::new(sizes[index]).expect("nonempty grouped tessellation upload"),
        });
        #[cfg(feature = "perf-diagnostics")]
        let write_started = Instant::now();
        // Every member of this group is consumed by the same first render
        // pass. One contiguous copy keeps the upload before that consumer while
        // avoiding six separate wgpu validation and Metal copy commands.
        self.has_active_staging_writes = true;
        let mut mapped = self.staging_belt.write_buffer(
            encoder,
            &destination,
            base_offset,
            NonZeroU64::new(write_size).expect("nonempty grouped tessellation upload"),
        );
        record_buffer_upload(write_size);
        mapped.slice(..).fill(0);
        for (request, offset) in requests.iter().zip(relative_offsets) {
            let start = usize::try_from(offset).expect("grouped upload offset fits usize");
            let end = start
                .checked_add(request.bytes.len())
                .expect("grouped upload range overflow");
            mapped.slice(start..end).copy_from_slice(request.bytes);
        }
        drop(mapped);
        #[cfg(feature = "perf-diagnostics")]
        {
            self.diagnostics.upload_calls = self.diagnostics.upload_calls.saturating_add(1);
            self.diagnostics.written_bytes =
                self.diagnostics.written_bytes.saturating_add(write_size);
            self.diagnostics.payload_bytes = self
                .diagnostics
                .payload_bytes
                .saturating_add(sizes.into_iter().sum());
            self.diagnostics.page_allocations = self
                .diagnostics
                .page_allocations
                .saturating_add((self.pages.len() - page_count_before) as u64);
            self.diagnostics.cpu_pack_ns = self
                .diagnostics
                .cpu_pack_ns
                .saturating_add(elapsed_ns(started));
            self.diagnostics.write_buffer_ns = self
                .diagnostics
                .write_buffer_ns
                .saturating_add(elapsed_ns(write_started));
        }
        uploads
    }

    fn finish_submission(&mut self, encoder: &wgpu::CommandEncoder) {
        #[cfg(feature = "perf-diagnostics")]
        {
            self.diagnostics.submissions = self.diagnostics.submissions.saturating_add(1);
        }
        for page in &mut self.pages {
            if page.used == 0 {
                continue;
            }
            #[cfg(feature = "perf-diagnostics")]
            {
                self.diagnostics.populated_pages =
                    self.diagnostics.populated_pages.saturating_add(1);
                self.diagnostics.used_bytes = self.diagnostics.used_bytes.saturating_add(page.used);
                self.diagnostics.populated_capacity_bytes = self
                    .diagnostics
                    .populated_capacity_bytes
                    .saturating_add(page.capacity);
            }
        }
        self.staging_belt.finish_and_recall_on_submit(encoder);
        // `finish_and_recall_on_submit` drains every active chunk into callbacks
        // owned by the encoder. If that encoder is dropped instead of submitted,
        // those chunks are dropped with it; only pre-finish active chunks need an
        // explicit belt reset on `TessellationUploadFrame::drop`.
        self.has_active_staging_writes = false;
    }

    fn discard_active_writes(&mut self) {
        if !self.has_active_staging_writes {
            return;
        }
        self.staging_belt = wgpu::util::StagingBelt::new(self.device.clone(), STAGING_CHUNK_SIZE);
        self.has_active_staging_writes = false;
        #[cfg(test)]
        {
            self.abandoned_staging_belt_resets =
                self.abandoned_staging_belt_resets.saturating_add(1);
        }
    }
}

struct UploadPage {
    buffer: wgpu::Buffer,
    capacity: u64,
    used: u64,
}

impl UploadPage {
    fn new(
        device: &wgpu::Device,
        label: &'static str,
        usage: wgpu::BufferUsages,
        capacity: u64,
    ) -> Self {
        Self {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: capacity,
                usage,
                mapped_at_creation: false,
            }),
            capacity,
            used: 0,
        }
    }
}

pub(crate) struct UploadSlice {
    buffer: wgpu::Buffer,
    offset: u64,
    size: NonZeroU64,
}

pub(crate) struct TessellationFlushResources {
    uniform_buffer: UploadSlice,
    path_buffer: UploadSlice,
    contour_buffer: UploadSlice,
}

impl TessellationFlushResources {
    pub(crate) fn uniform_binding(&self) -> wgpu::BindingResource<'_> {
        self.uniform_buffer.binding()
    }

    pub(crate) fn path_binding(&self) -> wgpu::BindingResource<'_> {
        self.path_buffer.binding()
    }

    pub(crate) fn contour_binding(&self) -> wgpu::BindingResource<'_> {
        self.contour_buffer.binding()
    }
}

pub(crate) struct TessellationEncoding {
    pub(crate) texture: Arc<TessellationTexture>,
    pub(crate) flush_resources: TessellationFlushResources,
}

pub(crate) struct TessellationEncodingWithPrefixes {
    pub(crate) tessellation: TessellationEncoding,
    pub(crate) prefixes: [UploadSlice; 2],
}

impl UploadSlice {
    pub(crate) fn binding(&self) -> wgpu::BindingResource<'_> {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &self.buffer,
            offset: self.offset,
            size: Some(self.size),
        })
    }

    pub(crate) fn slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer
            .slice(self.offset..self.offset + self.size.get())
    }
}

fn upload_capacity(required: u64) -> u64 {
    align_u64(
        required
            .max(MIN_UPLOAD_CAPACITY)
            .saturating_mul(5)
            .div_ceil(4),
        wgpu::COPY_BUFFER_ALIGNMENT,
    )
}

fn align_u64(value: u64, alignment: u64) -> u64 {
    debug_assert!(alignment.is_power_of_two());
    value.saturating_add(alignment - 1) & !(alignment - 1)
}

fn upload_group_layout<const N: usize>(sizes: [u64; N], alignments: [u64; N]) -> ([u64; N], u64) {
    let mut offsets = [0; N];
    let mut end = 0u64;
    for index in 0..N {
        assert!(sizes[index] != 0);
        end = align_u64(end, alignments[index].max(wgpu::COPY_BUFFER_ALIGNMENT));
        offsets[index] = end;
        end = end
            .checked_add(sizes[index])
            .expect("grouped tessellation upload layout overflow");
    }
    (offsets, align_u64(end, wgpu::COPY_BUFFER_ALIGNMENT))
}

fn frame_upload_group_layout(
    payloads: &[FrameUploadPayload<'_>],
    storage_alignment: u64,
) -> (Vec<u64>, u64, u64) {
    assert!(!payloads.is_empty() && payloads.iter().all(|payload| payload.size() != 0));
    let mut offsets = Vec::with_capacity(payloads.len());
    let mut end = 0u64;
    let mut group_alignment = wgpu::COPY_BUFFER_ALIGNMENT;
    for payload in payloads {
        let alignment = payload
            .alignment(storage_alignment)
            .max(wgpu::COPY_BUFFER_ALIGNMENT);
        group_alignment = group_alignment.max(alignment);
        end = align_u64(end, alignment);
        offsets.push(end);
        end = end
            .checked_add(payload.size())
            .expect("grouped frame upload layout overflow");
    }
    (
        offsets,
        align_u64(end, wgpu::COPY_BUFFER_ALIGNMENT),
        group_alignment,
    )
}

#[cfg(feature = "perf-diagnostics")]
fn elapsed_ns(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn msaa_packing_scratch_clears_stale_data_and_reuses_capacity() {
        let mut scratch = MsaaPackingScratch::default();
        scratch.begin_logical_flush(4);
        scratch.spans.push(TessVertexSpan::zeroed());
        scratch.contours.push(ContourData::zeroed());
        scratch.local_contour_ids.push(7);
        scratch.paths.push(PathData::zeroed());
        scratch.paints.push(crate::gpu::PaintData::zeroed());
        scratch.paint_aux.push(crate::gpu::PaintAuxData::zeroed());
        let capacities = scratch.capacities();

        scratch.begin_logical_flush(2);

        assert!(scratch.spans.is_empty());
        assert!(scratch.contours.is_empty());
        assert!(scratch.local_contour_ids.is_empty());
        assert_eq!(scratch.paths.len(), 1);
        assert_eq!(scratch.paints.len(), 1);
        assert_eq!(scratch.paint_aux.len(), 1);
        assert!(bytemuck::bytes_of(&scratch.paths[0])
            .iter()
            .all(|byte| *byte == 0));
        assert!(bytemuck::bytes_of(&scratch.paints[0])
            .iter()
            .all(|byte| *byte == 0));
        assert!(bytemuck::bytes_of(&scratch.paint_aux[0])
            .iter()
            .all(|byte| *byte == 0));
        assert_eq!(scratch.capacities(), capacities);
    }

    #[test]
    fn msaa_packing_scratch_bounds_retained_capacity_at_submission_completion() {
        let mut scratch = MsaaPackingScratch::default();
        scratch.spans.reserve_exact(
            MAX_RETAINED_MSAA_PACKING_SCRATCH_BYTES / size_of::<TessVertexSpan>() + 1,
        );
        assert!(scratch.retained_capacity_bytes() > MAX_RETAINED_MSAA_PACKING_SCRATCH_BYTES);

        scratch.finish_submission();

        assert!(scratch.spans.is_empty());
        assert!(scratch.contours.is_empty());
        assert!(scratch.local_contour_ids.is_empty());
        assert!(scratch.paths.is_empty());
        assert!(scratch.paints.is_empty());
        assert!(scratch.paint_aux.is_empty());
        assert!(scratch.retained_capacity_bytes() <= MAX_RETAINED_MSAA_PACKING_SCRATCH_BYTES);
    }

    #[test]
    fn abandoned_upload_frame_clears_and_bounds_msaa_packing_scratch() {
        let factory = crate::WgpuFactory::new(2, 2).unwrap();
        let uploads = factory
            .context
            .tessellator
            .begin_frame_uploads(&factory.context.device);
        let scratch = uploads.msaa_packing_scratch();
        {
            let mut scratch = scratch
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            scratch.begin_logical_flush(1);
            scratch.spans.reserve_exact(
                MAX_RETAINED_MSAA_PACKING_SCRATCH_BYTES / size_of::<TessVertexSpan>() + 1,
            );
            scratch.spans.push(TessVertexSpan::zeroed());
            assert!(scratch.retained_capacity_bytes() > MAX_RETAINED_MSAA_PACKING_SCRATCH_BYTES);
        }

        drop(uploads);

        let scratch = scratch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(scratch.spans.is_empty());
        assert!(scratch.contours.is_empty());
        assert!(scratch.local_contour_ids.is_empty());
        assert!(scratch.paths.is_empty());
        assert!(scratch.paints.is_empty());
        assert!(scratch.paint_aux.is_empty());
        assert!(scratch.retained_capacity_bytes() <= MAX_RETAINED_MSAA_PACKING_SCRATCH_BYTES);
    }

    #[test]
    fn repeated_abandoned_upload_frames_reset_active_staging_belts_and_recover() {
        let factory = crate::WgpuFactory::new(2, 2).unwrap();

        for frame_index in 0..BUFFER_RING_SIZE * 2 {
            let slot_index = frame_index % BUFFER_RING_SIZE;
            let mut encoder =
                factory
                    .context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("nuxie-abandoned-upload-test-encoder"),
                    });
            let mut uploads = factory
                .context
                .tessellator
                .begin_frame_uploads(&factory.context.device);
            let resets_before = uploads.slot.uploads.abandoned_staging_belt_resets;
            uploads.upload_uniforms(
                &factory.context.device,
                &mut encoder,
                &[u8::try_from(frame_index).unwrap(); 4],
            );
            assert!(uploads.slot.uploads.has_active_staging_writes);

            drop(uploads);
            drop(encoder);

            let slot = factory.context.tessellator.upload_slots[slot_index]
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            assert!(!slot.uploads.has_active_staging_writes);
            assert_eq!(
                slot.uploads.abandoned_staging_belt_resets,
                resets_before + 1
            );
        }

        let mut encoder =
            factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-recovered-upload-test-encoder"),
                });
        let mut uploads = factory
            .context
            .tessellator
            .begin_frame_uploads(&factory.context.device);
        let resets_before = uploads.slot.uploads.abandoned_staging_belt_resets;
        let expected = [9u8, 8, 7, 6];
        uploads.upload_uniforms(&factory.context.device, &mut encoder, &expected);
        uploads.finish_submission(&encoder);
        factory.context.queue.submit(Some(encoder.finish()));
        factory
            .context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        drop(uploads);

        let slot = factory.context.tessellator.upload_slots[0]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(!slot.uploads.has_active_staging_writes);
        assert_eq!(slot.uploads.abandoned_staging_belt_resets, resets_before);
    }

    #[test]
    fn next_submission_abandonment_resets_only_its_pending_staging_writes() {
        let factory = crate::WgpuFactory::new(2, 2).unwrap();
        let mut uploads = factory
            .context
            .tessellator
            .begin_frame_uploads(&factory.context.device);
        let resets_before = uploads.slot.uploads.abandoned_staging_belt_resets;
        let mut submitted_encoder =
            factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-completed-upload-test-encoder"),
                });
        uploads.upload_uniforms(
            &factory.context.device,
            &mut submitted_encoder,
            &[1, 2, 3, 4],
        );
        assert!(uploads.slot.uploads.has_active_staging_writes);
        uploads.finish_submission(&submitted_encoder);
        assert!(!uploads.slot.uploads.has_active_staging_writes);
        assert_eq!(
            uploads.slot.uploads.abandoned_staging_belt_resets,
            resets_before
        );
        factory
            .context
            .queue
            .submit(Some(submitted_encoder.finish()));
        factory
            .context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();

        uploads.begin_next_submission(&factory.context.device);
        let mut abandoned_encoder =
            factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-next-abandoned-upload-test-encoder"),
                });
        uploads.upload_uniforms(
            &factory.context.device,
            &mut abandoned_encoder,
            &[5, 6, 7, 8],
        );
        assert!(uploads.slot.uploads.has_active_staging_writes);

        drop(uploads);
        drop(abandoned_encoder);

        let slot = factory.context.tessellator.upload_slots[0]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(!slot.uploads.has_active_staging_writes);
        assert_eq!(
            slot.uploads.abandoned_staging_belt_resets,
            resets_before + 1
        );
    }

    #[test]
    fn completed_upload_frame_drop_keeps_its_clean_staging_belt() {
        let factory = crate::WgpuFactory::new(2, 2).unwrap();
        let mut uploads = factory
            .context
            .tessellator
            .begin_frame_uploads(&factory.context.device);
        let resets_before = uploads.slot.uploads.abandoned_staging_belt_resets;
        let mut encoder =
            factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-clean-upload-drop-test-encoder"),
                });
        uploads.upload_uniforms(&factory.context.device, &mut encoder, &[1, 2, 3, 4]);
        uploads.finish_submission(&encoder);
        factory.context.queue.submit(Some(encoder.finish()));
        factory
            .context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();

        drop(uploads);

        let slot = factory.context.tessellator.upload_slots[0]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(!slot.uploads.has_active_staging_writes);
        assert_eq!(slot.uploads.abandoned_staging_belt_resets, resets_before);
    }

    #[cfg(feature = "perf-counters")]
    #[test]
    fn upload_work_metrics_count_each_staging_write_and_exact_copy_bytes() {
        let factory = crate::WgpuFactory::new(2, 2).unwrap();
        let mut encoder =
            factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-upload-work-metrics-test-encoder"),
                });
        let recorder = crate::work_metrics::FrameWorkRecorder::new(true);
        let mut uploads = factory
            .context
            .tessellator
            .begin_frame_uploads(&factory.context.device);

        let single = [1u8, 2, 3];
        uploads.upload_uniforms(&factory.context.device, &mut encoder, &single);
        let single_write_size = align_u64(single.len() as u64, wgpu::COPY_BUFFER_ALIGNMENT);

        let storage = [4u8; 3];
        let vertices = [5u8; 5];
        let payloads = [
            FrameUploadPayload::Storage(&storage),
            FrameUploadPayload::Vertex(&vertices),
        ];
        let (_, frame_group_write_size, _) =
            frame_upload_group_layout(&payloads, uploads.slot.storage_alignment);
        uploads.upload_group(&factory.context.device, &mut encoder, &payloads);

        let grouped_a = [6u8; 3];
        let grouped_b = [7u8; 5];
        let requests = [
            UploadRequest::new(&grouped_a, uploads.slot.storage_alignment),
            UploadRequest::new(&grouped_b, wgpu::COPY_BUFFER_ALIGNMENT),
        ];
        let (_, const_group_write_size) = upload_group_layout(
            requests.map(|request| request.bytes.len() as u64),
            requests.map(|request| request.alignment),
        );
        uploads
            .slot
            .uploads
            .upload_group(&factory.context.device, &mut encoder, requests);

        uploads.finish_submission(&encoder);
        let metrics = recorder.snapshot();
        assert_eq!(metrics.buffer_upload_calls, 3);
        assert_eq!(
            metrics.buffer_upload_bytes,
            single_write_size + frame_group_write_size + const_group_write_size
        );
        #[cfg(feature = "perf-diagnostics")]
        {
            let diagnostics = uploads.diagnostics();
            assert_eq!(diagnostics.upload_calls, 3);
            assert_eq!(diagnostics.payload_bytes, 19);
            assert_eq!(
                diagnostics.written_bytes,
                single_write_size + frame_group_write_size + const_group_write_size
            );
            assert!(diagnostics.used_bytes > diagnostics.written_bytes);
        }

        factory.context.queue.submit(Some(encoder.finish()));
        factory
            .context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
    }

    #[cfg(feature = "perf-diagnostics")]
    #[test]
    fn msaa_packing_scratch_reports_capacity_reuse_and_retention() {
        let mut scratch = MsaaPackingScratch::default();
        scratch.begin_logical_flush(3);
        scratch.spans.push(TessVertexSpan::zeroed());
        scratch.contours.push(ContourData::zeroed());
        scratch.finish_logical_flush();
        let first_capacity = scratch.retained_capacity_bytes();

        scratch.begin_logical_flush(2);
        scratch.spans.push(TessVertexSpan::zeroed());
        scratch.contours.push(ContourData::zeroed());
        scratch.finish_logical_flush();

        assert_eq!(scratch.diagnostics.flushes, 2);
        assert_eq!(scratch.diagnostics.capacity_growths, 1);
        assert_eq!(
            scratch.diagnostics.peak_capacity_bytes,
            u64::try_from(first_capacity).unwrap()
        );
        scratch.finish_submission();
        assert_eq!(scratch.retained_capacity_bytes(), first_capacity);
        assert_eq!(scratch.diagnostics.released_capacity_bytes, 0);
    }

    #[test]
    fn upload_offsets_round_up_to_binding_alignment() {
        assert_eq!(align_u64(0, 4), 0);
        assert_eq!(align_u64(1, 4), 4);
        assert_eq!(align_u64(4, 4), 4);
        assert_eq!(align_u64(5, 4), 8);
        assert_eq!(align_u64(257, 256), 512);
    }

    #[test]
    fn upload_capacity_has_a_floor_and_headroom() {
        assert_eq!(upload_capacity(0), 5 * 1024);
        assert_eq!(upload_capacity(MIN_UPLOAD_CAPACITY), 5 * 1024);
        assert_eq!(upload_capacity(MIN_UPLOAD_CAPACITY + 1), 5 * 1024 + 4);
        assert_eq!(upload_capacity(8 * 1024), 10 * 1024);
    }

    #[test]
    fn grouped_uploads_preserve_each_binding_alignment() {
        let (offsets, size) = upload_group_layout([3, 5, 7], [256, 4, 256]);
        assert_eq!(offsets, [0, 4, 256]);
        assert_eq!(size, 264);
    }

    #[test]
    fn grouped_frame_uploads_align_storage_and_vertex_slices_independently() {
        let storage_a = [0; 3];
        let vertices = [0; 5];
        let storage_b = [0; 7];
        let payloads = [
            FrameUploadPayload::Storage(&storage_a),
            FrameUploadPayload::Vertex(&vertices),
            FrameUploadPayload::Storage(&storage_b),
        ];
        let (offsets, write_size, group_alignment) = frame_upload_group_layout(&payloads, 256);

        assert_eq!(offsets, [0, 4, 256]);
        assert_eq!(write_size, 264);
        assert_eq!(group_alignment, 256);
    }
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
