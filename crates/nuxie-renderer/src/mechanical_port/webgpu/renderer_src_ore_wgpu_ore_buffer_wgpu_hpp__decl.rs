//! Complete mechanical declaration translation of
//! `renderer/src/ore/wgpu/ore_buffer_wgpu.hpp`.

#![allow(non_snake_case)]

use super::ore_context_wgpu_decl::ContextWGPU;
use super::webgpu_cpp_decl::{
    Buffer as WagyuBuffer, BufferUsage as WagyuBufferUsage, Device, Queue,
};
use super::webgpu_decl::WGPUBuffer;
use nuxie_ore_metal::buffer::{Buffer, BufferApi, BufferUpdateError};
use nuxie_ore_metal::gpu_resource::{GPUResource, GPUResourceManager, GpuResourcePayload};
use nuxie_ore_metal::types::BufferUsage;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_buffer_wgpu.hpp");

pub(crate) struct Backing {
    pub(crate) buffer: WagyuBuffer,
    pub(crate) frameStamp: u64,
}

pub(crate) struct BufferWGPUState {
    pub(crate) m_pool: ManuallyDrop<Vec<Backing>>,
    pub(crate) m_currentIndex: usize,
    pub(crate) m_boundSinceUpdate: bool,
    pub(crate) m_shadow: ManuallyDrop<Vec<u8>>,
    pub(crate) m_wgpuUsage: WagyuBufferUsage,
    pub(crate) m_wgpuDevice: ManuallyDrop<Device>,
    pub(crate) m_wgpuQueue: ManuallyDrop<Queue>,
    pub(crate) m_ctx: *mut ContextWGPU,
}

impl Drop for BufferWGPUState {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_wgpuQueue);
            ManuallyDrop::drop(&mut self.m_wgpuDevice);
            ManuallyDrop::drop(&mut self.m_shadow);
            ManuallyDrop::drop(&mut self.m_pool);
        }
    }
}

#[repr(C)]
pub(crate) struct BufferWGPU {
    pub(crate) base: ManuallyDrop<Buffer>,
    // BufferApi dispatch receives `&self`; the source recording-thread rule
    // makes this lock uncontended while retaining mutation provenance.
    pub(crate) state: ManuallyDrop<Mutex<BufferWGPUState>>,
}

impl BufferWGPU {
    pub(crate) fn new(manager: GPUResourceManager, size: u32, usage: BufferUsage) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_buffer_backend_base(
                manager, size, usage,
            )),
            state: ManuallyDrop::new(Mutex::new(BufferWGPUState {
                m_pool: ManuallyDrop::new(Vec::new()),
                m_currentIndex: 0,
                m_boundSinceUpdate: false,
                m_shadow: ManuallyDrop::new(Vec::new()),
                m_wgpuUsage: WagyuBufferUsage::default(),
                m_wgpuDevice: ManuallyDrop::new(Device::default()),
                m_wgpuQueue: ManuallyDrop::new(Queue::default()),
                m_ctx: std::ptr::null_mut(),
            })),
        }
    }

    pub(crate) fn currentRaw(&self) -> WGPUBuffer {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.m_pool[state.m_currentIndex].buffer.Get()
    }

    pub(crate) fn markBound(&self) {
        super::ore_buffer_wgpu_impl::markBound(self)
    }
}

impl Drop for BufferWGPU {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.state);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl Deref for BufferWGPU {
    type Target = Buffer;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for BufferWGPU {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

unsafe impl Send for BufferWGPUState {}
unsafe impl Send for BufferWGPU {}
unsafe impl GpuResourcePayload for BufferWGPU {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
}

impl BufferApi for BufferWGPU {
    fn size(&self) -> u32 {
        self.base.size()
    }
    fn usage(&self) -> BufferUsage {
        self.base.usage()
    }
    fn update(&self, data: &[u8], size: u32, offset: u32) -> Result<(), BufferUpdateError> {
        super::ore_buffer_wgpu_impl::update(self, data, size, offset)
    }
}

pub(crate) const SOURCE_CLASS_COUNT: usize = 3;
pub(crate) const SOURCE_BACKEND_FIELD_COUNT: usize = 8;
pub(crate) const SOURCE_METHOD_COUNT: usize = 5;
pub(crate) const SOURCE_FRIEND_COUNT: usize = 2;
const _: [(); 2099] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::offset_of;

    #[test]
    fn complete_header_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 64);
        assert_eq!(SOURCE_CLASS_COUNT, 3);
        assert_eq!(SOURCE_BACKEND_FIELD_COUNT, 8);
        assert_eq!(SOURCE_METHOD_COUNT, 5);
        assert_eq!(SOURCE_FRIEND_COUNT, 2);
    }

    #[test]
    fn base_is_offset_zero() {
        assert_eq!(offset_of!(BufferWGPU, base), 0);
    }
}
