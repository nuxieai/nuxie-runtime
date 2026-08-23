//! Complete mechanical declaration translation of
//! `renderer/src/ore/wgpu/ore_bind_group_wgpu.hpp`.

#![allow(non_snake_case)]

use super::ore_buffer_wgpu_decl::BufferWGPU;
use super::ore_context_wgpu_decl::ContextWGPU;
use super::webgpu_cpp_decl::{
    BindGroup as WagyuBindGroup, BindGroupLayout as WagyuBindGroupLayout, Sampler as WagyuSampler,
    TextureView as WagyuTextureView,
};
use super::webgpu_decl::WGPUBuffer;
use nuxie_ore_metal::bind_group::BindGroup;
use nuxie_ore_metal::gpu_resource::{GPUResource, GPUResourceManager, GpuResourcePayload};
use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_bind_group_wgpu.hpp");

pub(crate) struct UBOEntry {
    pub(crate) buffer: NonNull<BufferWGPU>,
    pub(crate) binding: u32,
    pub(crate) offset: u64,
    pub(crate) size: u64,
}

pub(crate) struct TexEntry {
    pub(crate) binding: u32,
    pub(crate) view: WagyuTextureView,
}

pub(crate) struct SampEntry {
    pub(crate) binding: u32,
    pub(crate) sampler: WagyuSampler,
}

pub(crate) struct CachedGroup {
    pub(crate) key: Vec<WGPUBuffer>,
    pub(crate) bindGroup: WagyuBindGroup,
}

/// Source recording-thread mutation cell. `AnyResourceHandle` already rejects
/// payload access from every other thread, matching the unsynchronized C++
/// vector while permitting resolve through an erased shared handle.
pub(crate) struct RecordingCell<T>(UnsafeCell<T>);

impl<T> RecordingCell<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }
    pub(crate) unsafe fn getMutOnRecordingThread(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

unsafe impl<T: Send> Send for RecordingCell<T> {}

#[repr(C)]
pub(crate) struct BindGroupWGPU {
    pub(crate) base: ManuallyDrop<BindGroup>,
    pub(crate) m_uboEntries: ManuallyDrop<Vec<UBOEntry>>,
    pub(crate) m_texEntries: ManuallyDrop<Vec<TexEntry>>,
    pub(crate) m_sampEntries: ManuallyDrop<Vec<SampEntry>>,
    pub(crate) m_wgpuBGL: ManuallyDrop<WagyuBindGroupLayout>,
    pub(crate) m_label: ManuallyDrop<String>,
    pub(crate) m_cache: ManuallyDrop<RecordingCell<Vec<CachedGroup>>>,
    pub(crate) m_nullBindGroup: ManuallyDrop<WagyuBindGroup>,
    /// Rust safety sidecar for the source base-class `m_context` raw pointer.
    pub(crate) m_ctx: *mut ContextWGPU,
}

impl BindGroupWGPU {
    pub(crate) fn new(manager: GPUResourceManager) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_bind_group_backend_base(manager)),
            m_uboEntries: ManuallyDrop::new(Vec::new()),
            m_texEntries: ManuallyDrop::new(Vec::new()),
            m_sampEntries: ManuallyDrop::new(Vec::new()),
            m_wgpuBGL: ManuallyDrop::new(WagyuBindGroupLayout::default()),
            m_label: ManuallyDrop::new(String::new()),
            m_cache: ManuallyDrop::new(RecordingCell::new(Vec::new())),
            m_nullBindGroup: ManuallyDrop::new(WagyuBindGroup::default()),
            m_ctx: std::ptr::null_mut(),
        }
    }

    pub(crate) fn resolveBindGroup(&self) -> &WagyuBindGroup {
        super::ore_bind_group_wgpu_impl::resolveBindGroup(self)
    }

    pub(crate) fn markUBOsBound(&self) {
        super::ore_bind_group_wgpu_impl::markUBOsBound(self)
    }
}

impl Drop for BindGroupWGPU {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_nullBindGroup);
            ManuallyDrop::drop(&mut self.m_cache);
            ManuallyDrop::drop(&mut self.m_label);
            ManuallyDrop::drop(&mut self.m_wgpuBGL);
            ManuallyDrop::drop(&mut self.m_sampEntries);
            ManuallyDrop::drop(&mut self.m_texEntries);
            ManuallyDrop::drop(&mut self.m_uboEntries);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl Deref for BindGroupWGPU {
    type Target = BindGroup;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BindGroupWGPU {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

unsafe impl Send for BindGroupWGPU {}
unsafe impl GpuResourcePayload for BindGroupWGPU {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
}

pub(crate) const SOURCE_CLASS_COUNT: usize = 6;
pub(crate) const SOURCE_BACKEND_FIELD_COUNT: usize = 7;
pub(crate) const SOURCE_METHOD_COUNT: usize = 4;
pub(crate) const SOURCE_FRIEND_COUNT: usize = 2;
const _: [(); 2063] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    #[test]
    fn complete_header_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 70);
        assert_eq!(SOURCE_CLASS_COUNT, 6);
        assert_eq!(SOURCE_BACKEND_FIELD_COUNT, 7);
        assert_eq!(SOURCE_METHOD_COUNT, 4);
        assert_eq!(SOURCE_FRIEND_COUNT, 2);
    }

    #[test]
    fn base_is_offset_zero_and_cached_keys_are_raw_handles() {
        assert_eq!(offset_of!(BindGroupWGPU, base), 0);
        assert_eq!(size_of::<WGPUBuffer>(), size_of::<usize>());
    }
}
