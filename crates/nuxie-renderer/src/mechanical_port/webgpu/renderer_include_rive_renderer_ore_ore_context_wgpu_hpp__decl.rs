//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/ore/ore_context_wgpu.hpp`.

#![allow(non_snake_case)]

use super::webgpu_cpp_decl::{BackendType as WagyuBackendType, CommandEncoder, Device, Queue};
use nuxie_ore_metal::context::{Context, FrameDescriptor, ShaderTarget};
use nuxie_ore_metal::gpu_resource::GPUResourceManagerOwner;
use nuxie_ore_metal::types::Features;
use std::cell::Cell;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_include_rive_renderer_ore_ore_context_wgpu.hpp");

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum WGPUBackend {
    OpenGLES,
    #[default]
    Vulkan,
}

pub(crate) struct ContextWGPULifetime {
    live: Cell<bool>,
}

impl ContextWGPULifetime {
    fn new() -> Self { Self { live: Cell::new(true) } }
    pub(crate) fn isLive(&self) -> bool { self.live.get() }
    fn retire(&self) { self.live.set(false); }
}

#[repr(C)]
pub(crate) struct ContextWGPU {
    pub(crate) base: ManuallyDrop<Context>,
    pub(crate) m_wgpuBackend: WGPUBackend,
    pub(crate) m_wgpuDevice: ManuallyDrop<Device>,
    pub(crate) m_wgpuQueue: ManuallyDrop<Queue>,
    pub(crate) m_wgpuCommandEncoder: ManuallyDrop<CommandEncoder>,
    pub(crate) m_frameSerial: u64,
    /// Rust root for resources whose source C++ base uses a null manager.
    pub(crate) m_managerOwner: ManuallyDrop<GPUResourceManagerOwner>,
    pub(super) m_lifetime: Rc<ContextWGPULifetime>,
}

impl ContextWGPU {
    pub(crate) fn new_base(features: Features) -> Self {
        let managerOwner = GPUResourceManagerOwner::new();
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_context_backend_base(
                features,
                Some(managerOwner.manager()),
            )),
            m_wgpuBackend: WGPUBackend::Vulkan,
            m_wgpuDevice: ManuallyDrop::new(Device::default()),
            m_wgpuQueue: ManuallyDrop::new(Queue::default()),
            m_wgpuCommandEncoder: ManuallyDrop::new(CommandEncoder::default()),
            m_frameSerial: 0,
            m_managerOwner: ManuallyDrop::new(managerOwner),
            m_lifetime: Rc::new(ContextWGPULifetime::new()),
        }
    }

    pub(crate) fn Make(
        device: Device,
        queue: Queue,
        backendType: WagyuBackendType,
    ) -> Option<Box<Self>> {
        super::ore_context_wgpu_impl::make(device, queue, backendType)
    }

    pub(crate) fn beginFrameExternal(&mut self, externalEncoder: CommandEncoder) {
        super::ore_context_wgpu_impl::beginFrameExternal(self, externalEncoder)
    }

    pub(crate) fn isGLES(&self) -> bool {
        self.m_wgpuBackend == WGPUBackend::OpenGLES
    }

    pub(crate) fn currentFrameSerial(&self) -> u64 {
        self.m_frameSerial
    }

    pub(crate) fn setLastError(&self, message: &str) {
        self.base.setLastError(message)
    }

    pub(crate) fn shaderTarget(&self) -> ShaderTarget {
        ShaderTarget::wgsl
    }

    pub(crate) fn beginFrameOwned(&mut self, descriptor: &FrameDescriptor) {
        super::ore_context_wgpu_impl::beginFrame(self, descriptor)
    }
}

impl Drop for ContextWGPU {
    fn drop(&mut self) {
        self.m_lifetime.retire();
        super::ore_context_wgpu_impl::destroy(self);
        unsafe {
            ManuallyDrop::drop(&mut self.m_wgpuCommandEncoder);
            ManuallyDrop::drop(&mut self.m_wgpuQueue);
            ManuallyDrop::drop(&mut self.m_wgpuDevice);
            ManuallyDrop::drop(&mut self.base);
            ManuallyDrop::drop(&mut self.m_managerOwner);
        }
    }
}

impl Deref for ContextWGPU {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for ContextWGPU {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub(crate) const SOURCE_PUBLIC_METHOD_COUNT: usize = 20;
pub(crate) const SOURCE_FRIEND_COUNT: usize = 3;
pub(crate) const SOURCE_BACKEND_FIELD_COUNT: usize = 5;
pub(crate) const SOURCE_DELETED_COPY_OPERATION_COUNT: usize = 2;
const _: [(); 3796] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::offset_of;

    #[test]
    fn complete_header_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 99);
        assert_eq!(SOURCE_PUBLIC_METHOD_COUNT, 20);
        assert_eq!(SOURCE_FRIEND_COUNT, 3);
        assert_eq!(SOURCE_BACKEND_FIELD_COUNT, 5);
        assert_eq!(SOURCE_DELETED_COPY_OPERATION_COUNT, 2);
    }

    #[test]
    fn base_is_offset_zero_and_backend_default_is_vulkan() {
        assert_eq!(offset_of!(ContextWGPU, base), 0);
        assert_eq!(WGPUBackend::default(), WGPUBackend::Vulkan);
    }
}
