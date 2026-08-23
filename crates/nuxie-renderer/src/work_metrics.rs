#[cfg(feature = "perf-counters")]
use std::cell::RefCell;
#[cfg(all(test, feature = "perf-counters"))]
use std::collections::HashMap;
use std::ops::{Deref, DerefMut, Range};
use wgpu::util::DeviceExt;
use crate::BackendWorkMetrics;

#[cfg(feature = "perf-counters")]
thread_local! {
    static ACTIVE: RefCell<Option<BackendWorkMetrics>> = const { RefCell::new(None) };
}

#[cfg(all(test, feature = "perf-counters"))]
thread_local! {
    static COUNTED_BUFFER_INIT_LABELS: RefCell<HashMap<String, u64>> = RefCell::default();
}

#[cfg(all(test, feature = "perf-counters"))]
pub(crate) fn reset_counted_buffer_init_labels() {
    COUNTED_BUFFER_INIT_LABELS.with(|labels| labels.borrow_mut().clear());
}

#[cfg(all(test, feature = "perf-counters"))]
pub(crate) fn counted_buffer_init_labels() -> Vec<(String, u64)> {
    COUNTED_BUFFER_INIT_LABELS.with(|labels| {
        let mut labels = labels
            .borrow()
            .iter()
            .map(|(label, count)| (label.clone(), *count))
            .collect::<Vec<_>>();
        labels.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        labels
    })
}

pub(crate) struct FrameWorkRecorder {
    enabled: bool,
}

impl FrameWorkRecorder {
    pub(crate) fn new(requested: bool) -> Self {
        let enabled = requested && cfg!(feature = "perf-counters");
        #[cfg(feature = "perf-counters")]
        if enabled {
            ACTIVE.with(|active| {
                let previous = active.replace(Some(BackendWorkMetrics::default()));
                assert!(
                    previous.is_none(),
                    "renderer work counters cannot be nested"
                );
            });
        }
        Self { enabled }
    }

    pub(crate) fn snapshot(&self) -> BackendWorkMetrics {
        if !self.enabled {
            return BackendWorkMetrics::default();
        }
        #[cfg(feature = "perf-counters")]
        {
            return ACTIVE.with(|active| active.borrow().unwrap_or_default());
        }
        #[allow(unreachable_code)]
        BackendWorkMetrics::default()
    }
}

impl Drop for FrameWorkRecorder {
    fn drop(&mut self) {
        #[cfg(feature = "perf-counters")]
        if self.enabled {
            ACTIVE.with(|active| {
                active.borrow_mut().take();
            });
        }
    }
}

#[inline]
fn record(update: impl FnOnce(&mut BackendWorkMetrics)) {
    #[cfg(feature = "perf-counters")]
    ACTIVE.with(|active| {
        if let Some(metrics) = active.borrow_mut().as_mut() {
            update(metrics);
        }
    });
    #[cfg(not(feature = "perf-counters"))]
    let _ = update;
}

pub(crate) fn record_buffer_upload(bytes: u64) {
    record(|metrics| {
        metrics.buffer_upload_calls = metrics.buffer_upload_calls.saturating_add(1);
        metrics.buffer_upload_bytes = metrics.buffer_upload_bytes.saturating_add(bytes);
    });
}

fn texture_binding_count(entries: &[wgpu::BindGroupEntry<'_>]) -> u64 {
    entries
        .iter()
        .map(|entry| match &entry.resource {
            wgpu::BindingResource::TextureView(_) => 1,
            wgpu::BindingResource::TextureViewArray(views) => views.len() as u64,
            _ => 0,
        })
        .sum()
}

pub(crate) trait CountedDeviceExt {
    fn create_counted_bind_group(
        &self,
        descriptor: &wgpu::BindGroupDescriptor<'_>,
    ) -> wgpu::BindGroup;
    fn create_counted_buffer_init(
        &self,
        descriptor: &wgpu::util::BufferInitDescriptor<'_>,
    ) -> wgpu::Buffer;
    fn create_counted_command_encoder(
        &self,
        descriptor: &wgpu::CommandEncoderDescriptor<'_>,
    ) -> wgpu::CommandEncoder;
}

impl CountedDeviceExt for wgpu::Device {
    fn create_counted_bind_group(
        &self,
        descriptor: &wgpu::BindGroupDescriptor<'_>,
    ) -> wgpu::BindGroup {
        let texture_bindings = texture_binding_count(descriptor.entries);
        record(|metrics| {
            metrics.bind_groups_created = metrics.bind_groups_created.saturating_add(1);
            metrics.texture_bindings = metrics.texture_bindings.saturating_add(texture_bindings);
        });
        self.create_bind_group(descriptor)
    }

    fn create_counted_buffer_init(
        &self,
        descriptor: &wgpu::util::BufferInitDescriptor<'_>,
    ) -> wgpu::Buffer {
        #[cfg(all(test, feature = "perf-counters"))]
        if let Some(label) = descriptor.label {
            COUNTED_BUFFER_INIT_LABELS.with(|labels| {
                let mut labels = labels.borrow_mut();
                *labels.entry(label.to_owned()).or_default() += 1;
            });
        }
        let bytes = descriptor.contents.len() as u64;
        record_buffer_upload(bytes);
        self.create_buffer_init(descriptor)
    }

    fn create_counted_command_encoder(
        &self,
        descriptor: &wgpu::CommandEncoderDescriptor<'_>,
    ) -> wgpu::CommandEncoder {
        record(|metrics| {
            metrics.command_encoders = metrics.command_encoders.saturating_add(1);
        });
        self.create_command_encoder(descriptor)
    }
}

pub(crate) trait CountedQueueExt {
    fn submit_counted<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        command_buffers: I,
    ) -> wgpu::SubmissionIndex;
    fn write_counted_texture(
        &self,
        texture: wgpu::TexelCopyTextureInfo<'_>,
        data: &[u8],
        data_layout: wgpu::TexelCopyBufferLayout,
        size: wgpu::Extent3d,
    );
}

impl CountedQueueExt for wgpu::Queue {
    fn submit_counted<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        command_buffers: I,
    ) -> wgpu::SubmissionIndex {
        record(|metrics| {
            metrics.queue_submissions = metrics.queue_submissions.saturating_add(1);
        });
        self.submit(command_buffers)
    }

    fn write_counted_texture(
        &self,
        texture: wgpu::TexelCopyTextureInfo<'_>,
        data: &[u8],
        data_layout: wgpu::TexelCopyBufferLayout,
        size: wgpu::Extent3d,
    ) {
        let bytes = data.len() as u64;
        record(|metrics| {
            metrics.texture_upload_calls = metrics.texture_upload_calls.saturating_add(1);
            metrics.texture_upload_bytes = metrics.texture_upload_bytes.saturating_add(bytes);
        });
        self.write_texture(texture, data, data_layout, size);
    }
}

pub(crate) struct CountedRenderPass<'encoder> {
    inner: wgpu::RenderPass<'encoder>,
}

pub(crate) trait CountedCommandEncoderExt {
    fn begin_counted_render_pass<'encoder>(
        &'encoder mut self,
        descriptor: &wgpu::RenderPassDescriptor<'_>,
    ) -> CountedRenderPass<'encoder>;
    #[allow(dead_code)]
    fn clear_counted_buffer(
        &mut self,
        buffer: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        size: Option<wgpu::BufferAddress>,
    );
}

impl CountedCommandEncoderExt for wgpu::CommandEncoder {
    fn begin_counted_render_pass<'encoder>(
        &'encoder mut self,
        descriptor: &wgpu::RenderPassDescriptor<'_>,
    ) -> CountedRenderPass<'encoder> {
        record(|metrics| {
            metrics.render_passes = metrics.render_passes.saturating_add(1);
        });
        CountedRenderPass {
            inner: self.begin_render_pass(descriptor),
        }
    }

    fn clear_counted_buffer(
        &mut self,
        buffer: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        size: Option<wgpu::BufferAddress>,
    ) {
        let bytes = size.unwrap_or_else(|| buffer.size().saturating_sub(offset));
        record(|metrics| {
            metrics.buffer_clear_calls = metrics.buffer_clear_calls.saturating_add(1);
            metrics.buffer_clear_bytes = metrics.buffer_clear_bytes.saturating_add(bytes);
        });
        self.clear_buffer(buffer, offset, size);
    }
}

impl CountedRenderPass<'_> {
    pub(crate) fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &wgpu::BindGroup,
        offsets: &[wgpu::DynamicOffset],
    ) {
        record(|metrics| {
            metrics.bind_group_sets = metrics.bind_group_sets.saturating_add(1);
        });
        self.inner.set_bind_group(index, bind_group, offsets);
    }

    pub(crate) fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        let instance_count = u64::from(instances.end.saturating_sub(instances.start));
        record(|metrics| {
            metrics.gpu_draw_calls = metrics.gpu_draw_calls.saturating_add(1);
            metrics.gpu_draw_instances = metrics.gpu_draw_instances.saturating_add(instance_count);
        });
        self.inner.draw(vertices, instances);
    }

    pub(crate) fn draw_indexed(
        &mut self,
        indices: Range<u32>,
        base_vertex: i32,
        instances: Range<u32>,
    ) {
        let instance_count = u64::from(instances.end.saturating_sub(instances.start));
        record(|metrics| {
            metrics.gpu_draw_calls = metrics.gpu_draw_calls.saturating_add(1);
            metrics.gpu_draw_instances = metrics.gpu_draw_instances.saturating_add(instance_count);
        });
        self.inner.draw_indexed(indices, base_vertex, instances);
    }

    pub(crate) fn draw_tessellation_spans(
        &mut self,
        indices: Range<u32>,
        base_vertex: i32,
        instances: Range<u32>,
    ) {
        let instance_count = u64::from(instances.end.saturating_sub(instances.start));
        record(|metrics| {
            metrics.tessellation_spans = metrics.tessellation_spans.saturating_add(instance_count);
        });
        self.draw_indexed(indices, base_vertex, instances);
    }

    pub(crate) fn draw_path_patches(
        &mut self,
        indices: Range<u32>,
        base_vertex: i32,
        instances: Range<u32>,
    ) {
        let instance_count = u64::from(instances.end.saturating_sub(instances.start));
        record(|metrics| {
            metrics.path_patches = metrics.path_patches.saturating_add(instance_count);
        });
        self.draw_indexed(indices, base_vertex, instances);
    }
}

impl<'encoder> Deref for CountedRenderPass<'encoder> {
    type Target = wgpu::RenderPass<'encoder>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CountedRenderPass<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
