//! GPU tessellation pass translated from `renderer/src/shaders/tessellate.glsl`.

use crate::gpu::{ContourData, FlushUniforms, PathData, TessVertexSpan};
use crate::work_metrics::{CountedCommandEncoderExt, CountedDeviceExt, CountedQueueExt};
#[cfg(feature = "perf-diagnostics")]
use std::time::Instant;
use std::{
    num::NonZeroU64,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, MutexGuard,
    },
};

// Mirrors C++ `gpu::kBufferRingSize`; the frame guard owns its slot through GPU completion.
const BUFFER_RING_SIZE: usize = 3;
const MIN_UPLOAD_CAPACITY: u64 = 4 * 1024;

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
}

pub(crate) struct Tessellator {
    pub pipeline: wgpu::RenderPipeline,
    pub flush_layout: wgpu::BindGroupLayout,
    pub span_indices: wgpu::Buffer,
    _linear_sampler: wgpu::Sampler,
    sampler_group: wgpu::BindGroup,
    upload_slots: [Mutex<TessellationUploadSlot>; BUFFER_RING_SIZE],
    next_upload_slot: AtomicUsize,
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
            upload_slots: std::array::from_fn(|_| Mutex::new(TessellationUploadSlot::new(&limits))),
            next_upload_slot: AtomicUsize::new(0),
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

    pub(crate) fn encode(
        &self,
        device: &wgpu::Device,
        uploads: &mut TessellationUploadFrame<'_>,
        encoder: &mut wgpu::CommandEncoder,
        feather_lut: &wgpu::TextureView,
        spans: &[TessVertexSpan],
        uniforms: &FlushUniforms,
        paths: &[PathData],
        contours: &[ContourData],
        height: u32,
    ) -> wgpu::Texture {
        assert!(!spans.is_empty() && !paths.is_empty() && !contours.is_empty());
        let span_buffer = uploads.upload_spans(device, bytemuck::cast_slice(spans));
        let uniform_buffer = uploads.upload_uniforms(device, bytemuck::bytes_of(uniforms));
        let path_buffer = uploads.upload_paths(device, bytemuck::cast_slice(paths));
        let contour_buffer = uploads.upload_contours(device, bytemuck::cast_slice(contours));
        let flush_group = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-tessellation-flush-group"),
            layout: &self.flush_layout,
            entries: &[
                binding(0, uniform_buffer.binding()),
                binding(3, path_buffer.binding()),
                binding(6, contour_buffer.binding()),
                binding(10, wgpu::BindingResource::TextureView(feather_lut)),
            ],
        });
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-tessellation-data"),
            size: wgpu::Extent3d {
                width: 2048,
                height: height.max(1),
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
        let mut pass = encoder.begin_counted_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("nuxie-tessellation-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
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
        pass.draw_tessellation_spans(0..12, 0, 0..spans.len() as u32);
        drop(pass);
        texture
    }
}

pub(crate) struct TessellationUploadFrame<'a> {
    slot: MutexGuard<'a, TessellationUploadSlot>,
}

impl TessellationUploadFrame<'_> {
    fn upload_spans(&mut self, device: &wgpu::Device, bytes: &[u8]) -> UploadSlice {
        self.slot
            .uploads
            .upload(device, bytes, wgpu::COPY_BUFFER_ALIGNMENT)
    }

    pub(crate) fn upload_uniforms(&mut self, device: &wgpu::Device, bytes: &[u8]) -> UploadSlice {
        let alignment = self.slot.uniform_alignment;
        self.slot.uploads.upload(device, bytes, alignment)
    }

    pub(crate) fn upload_storage(&mut self, device: &wgpu::Device, bytes: &[u8]) -> UploadSlice {
        let alignment = self.slot.storage_alignment;
        self.slot.uploads.upload(device, bytes, alignment)
    }

    fn upload_paths(&mut self, device: &wgpu::Device, bytes: &[u8]) -> UploadSlice {
        let alignment = self.slot.storage_alignment;
        self.slot.uploads.upload(device, bytes, alignment)
    }

    fn upload_contours(&mut self, device: &wgpu::Device, bytes: &[u8]) -> UploadSlice {
        let alignment = self.slot.storage_alignment;
        self.slot.uploads.upload(device, bytes, alignment)
    }

    pub(crate) fn flush(&mut self, queue: &wgpu::Queue) {
        self.slot.flush(queue);
    }

    pub(crate) fn begin_next_submission(&mut self, device: &wgpu::Device) {
        self.slot.begin_submission(device);
    }

    #[cfg(feature = "perf-diagnostics")]
    pub(crate) fn diagnostics(&self) -> TessellationUploadDiagnostics {
        self.slot.uploads.diagnostics
    }
}

struct TessellationUploadSlot {
    uploads: UploadArena,
    uniform_alignment: u64,
    storage_alignment: u64,
}

impl TessellationUploadSlot {
    fn new(limits: &wgpu::Limits) -> Self {
        Self {
            uploads: UploadArena::new(
                "nuxie-tessellation-upload-ring",
                wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::STORAGE,
            ),
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
    }

    fn flush(&mut self, queue: &wgpu::Queue) {
        self.uploads.flush(queue);
    }
}

struct UploadArena {
    label: &'static str,
    usage: wgpu::BufferUsages,
    pages: Vec<UploadPage>,
    #[cfg(feature = "perf-diagnostics")]
    diagnostics: TessellationUploadDiagnostics,
}

impl UploadArena {
    fn new(label: &'static str, usage: wgpu::BufferUsages) -> Self {
        Self {
            label,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            pages: Vec::new(),
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
            page.shadow.clear();
        }
    }

    fn upload(&mut self, device: &wgpu::Device, bytes: &[u8], alignment: u64) -> UploadSlice {
        #[cfg(feature = "perf-diagnostics")]
        let started = Instant::now();
        assert!(!bytes.is_empty());
        let size = bytes.len() as u64;
        let alignment = alignment.max(wgpu::COPY_BUFFER_ALIGNMENT);
        #[cfg(feature = "perf-diagnostics")]
        let page_count_before = self.pages.len();
        let page_index = self
            .pages
            .last()
            .and_then(|page| {
                let offset = align_u64(page.used, alignment);
                (offset.saturating_add(size) <= page.capacity).then_some(self.pages.len() - 1)
            })
            .unwrap_or_else(|| {
                let previous_capacity = self.pages.last().map_or(0, |page| page.capacity);
                self.pages.push(UploadPage::new(
                    device,
                    self.label,
                    self.usage,
                    upload_capacity(size.max(previous_capacity)),
                ));
                self.pages.len() - 1
            });
        let upload = {
            let page = &mut self.pages[page_index];
            let offset = align_u64(page.used, alignment);
            let end = offset
                .checked_add(size)
                .expect("tessellation upload overflow");
            page.shadow.resize(end as usize, 0);
            page.shadow[offset as usize..end as usize].copy_from_slice(bytes);
            page.used = end;
            UploadSlice {
                buffer: page.buffer.clone(),
                offset,
                size: NonZeroU64::new(size).expect("nonempty tessellation upload"),
            }
        };
        #[cfg(feature = "perf-diagnostics")]
        {
            self.diagnostics.upload_calls = self.diagnostics.upload_calls.saturating_add(1);
            self.diagnostics.payload_bytes = self.diagnostics.payload_bytes.saturating_add(size);
            self.diagnostics.page_allocations = self
                .diagnostics
                .page_allocations
                .saturating_add((self.pages.len() - page_count_before) as u64);
            self.diagnostics.cpu_pack_ns = self
                .diagnostics
                .cpu_pack_ns
                .saturating_add(elapsed_ns(started));
        }
        upload
    }

    fn flush(&mut self, queue: &wgpu::Queue) {
        #[cfg(feature = "perf-diagnostics")]
        {
            self.diagnostics.submissions = self.diagnostics.submissions.saturating_add(1);
        }
        for page in &mut self.pages {
            if page.used == 0 {
                continue;
            }
            let write_size = align_u64(page.used, wgpu::COPY_BUFFER_ALIGNMENT);
            page.shadow.resize(write_size as usize, 0);
            #[cfg(feature = "perf-diagnostics")]
            {
                self.diagnostics.populated_pages =
                    self.diagnostics.populated_pages.saturating_add(1);
                self.diagnostics.used_bytes = self.diagnostics.used_bytes.saturating_add(page.used);
                self.diagnostics.written_bytes =
                    self.diagnostics.written_bytes.saturating_add(write_size);
                self.diagnostics.populated_capacity_bytes = self
                    .diagnostics
                    .populated_capacity_bytes
                    .saturating_add(page.capacity);
            }
            #[cfg(feature = "perf-diagnostics")]
            let started = Instant::now();
            queue.write_counted_buffer(&page.buffer, 0, &page.shadow);
            #[cfg(feature = "perf-diagnostics")]
            {
                self.diagnostics.write_buffer_ns = self
                    .diagnostics
                    .write_buffer_ns
                    .saturating_add(elapsed_ns(started));
            }
        }
    }
}

struct UploadPage {
    buffer: wgpu::Buffer,
    capacity: u64,
    used: u64,
    shadow: Vec<u8>,
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
            shadow: Vec::new(),
        }
    }
}

pub(crate) struct UploadSlice {
    buffer: wgpu::Buffer,
    offset: u64,
    size: NonZeroU64,
}

impl UploadSlice {
    pub(crate) fn binding(&self) -> wgpu::BindingResource<'_> {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &self.buffer,
            offset: self.offset,
            size: Some(self.size),
        })
    }

    fn slice(&self) -> wgpu::BufferSlice<'_> {
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

#[cfg(feature = "perf-diagnostics")]
fn elapsed_ns(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

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
