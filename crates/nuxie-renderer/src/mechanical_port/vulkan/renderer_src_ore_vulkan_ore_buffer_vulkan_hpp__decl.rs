//! Complete mechanical declaration translation of
//! `renderer/src/ore/vulkan/ore_buffer_vulkan.hpp`.

#![allow(non_snake_case)]

use super::vulkan_context_decl::VulkanContext;
use ash::vk;
use ash::vk::Handle;
use nuxie_ore_metal::buffer::{Buffer, BufferApi};
use nuxie_ore_metal::gpu_resource::{GPUResource, GPUResourceManager, GpuResourcePayload};
use nuxie_ore_metal::types::BufferUsage;
use std::cell::{Cell, UnsafeCell};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Rust ownership form of the source `VmaAllocation` handle mirror.
///
/// A pooled allocation is owned by its stable `Box` in `Backing` and aliased
/// here. A staging allocation is owned directly here because the source leaves
/// `m_pool` empty for staging buffers. This preserves the source's one native
/// allocation and exactly-once destruction without duplicating the non-`Copy`
/// `vk_mem::Allocation` wrapper.
pub(super) enum AllocationMirror {
    Null,
    Owned(Box<vk_mem::Allocation>),
    Alias(*mut vk_mem::Allocation),
}

impl AllocationMirror {
    pub(super) fn isNull(&self) -> bool {
        matches!(self, Self::Null)
    }
}

// struct Backing
#[repr(C)]
pub(crate) struct Backing {
    pub(crate) vkBuffer: vk::Buffer,
    pub(super) vmaAllocation: Option<Box<vk_mem::Allocation>>,
    pub(crate) mappedPtr: *mut u8,
    pub(crate) frameStamp: u64,
}

impl Default for Backing {
    fn default() -> Self {
        Self {
            vkBuffer: vk::Buffer::null(),
            vmaAllocation: None,
            mappedPtr: core::ptr::null_mut(),
            frameStamp: 0,
        }
    }
}

// class BufferVulkan : public LITE_RTTI_OVERRIDE(Buffer, BufferVulkan)
#[repr(C)]
pub(crate) struct BufferVulkan {
    pub(crate) base: ManuallyDrop<Buffer>,
    pub(crate) m_vkBuffer: Cell<vk::Buffer>,
    pub(super) m_vmaAllocation: UnsafeCell<AllocationMirror>,
    pub(crate) m_vkMappedPtr: Cell<*mut u8>,
    pub(crate) m_vkDevice: vk::Device,
    pub(super) m_vk: ManuallyDrop<UnsafeCell<Option<Arc<VulkanContext>>>>,
    pub(super) m_pool: ManuallyDrop<UnsafeCell<Vec<Backing>>>,
    pub(crate) m_currentIndex: Cell<usize>,
    pub(crate) m_boundSinceUpdate: Cell<bool>,
    pub(crate) m_vkUsage: vk::BufferUsageFlags,
}

unsafe impl Send for BufferVulkan {}

impl BufferVulkan {
    // BufferVulkan(rcp<GPUResourceManager> manager, uint32_t size,
    //              BufferUsage usage)
    pub(crate) fn new(manager: GPUResourceManager, size: u32, usage: BufferUsage) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_buffer_backend_base(
                manager, size, usage,
            )),
            m_vkBuffer: Cell::new(vk::Buffer::null()),
            m_vmaAllocation: UnsafeCell::new(AllocationMirror::Null),
            m_vkMappedPtr: Cell::new(core::ptr::null_mut()),
            m_vkDevice: vk::Device::null(),
            m_vk: ManuallyDrop::new(UnsafeCell::new(None)),
            m_pool: ManuallyDrop::new(UnsafeCell::new(Vec::new())),
            m_currentIndex: Cell::new(0),
            m_boundSinceUpdate: Cell::new(false),
            m_vkUsage: vk::BufferUsageFlags::empty(),
        }
    }

    // VkBuffer current() const { return m_vkBuffer; }
    pub(crate) fn current(&self) -> vk::Buffer {
        self.m_vkBuffer.get()
    }

    pub(crate) fn mappedPtr(&self) -> *mut u8 {
        self.m_vkMappedPtr.get()
    }

    pub(super) fn vk(&self) -> &VulkanContext {
        // SAFETY: ContextVulkan/TextureVulkan publish `m_vk` before any native
        // allocation is installed or any source operation can reach this object.
        unsafe { (&*self.m_vk.get()).as_ref() }
            .expect("BufferVulkan requires its source VulkanContext retain")
    }

    pub(super) fn manager(&self) -> &GPUResourceManager {
        self.base
            .gpu_resource()
            .manager()
            .expect("BufferVulkan requires its source GPUResourceManager retain")
    }

    /// Source friend access used by ContextVulkan and TextureVulkan.
    pub(crate) fn setVulkanContext(&mut self, vk: Arc<VulkanContext>) {
        *self.m_vk.get_mut() = Some(vk);
    }

    /// Source friend access used by ContextVulkan.
    pub(crate) fn setDeviceAndUsage(
        &mut self,
        device: vk::Device,
        usage: vk::BufferUsageFlags,
    ) {
        self.m_vkDevice = device;
        self.m_vkUsage = usage;
    }

    /// Seeds the source backing pool with its first VMA allocation.
    pub(crate) fn installPooledBacking(
        &mut self,
        buffer: vk::Buffer,
        allocation: vk_mem::Allocation,
        mapped: *mut u8,
    ) {
        debug_assert!(self.m_vkBuffer.get().is_null());
        debug_assert!(self.m_pool.get_mut().is_empty());
        let mut allocation = Box::new(allocation);
        let alias = (&mut *allocation) as *mut vk_mem::Allocation;
        self.m_pool.get_mut().push(Backing {
            vkBuffer: buffer,
            vmaAllocation: Some(allocation),
            mappedPtr: mapped,
            frameStamp: 0,
        });
        *self.m_vmaAllocation.get_mut() = AllocationMirror::Alias(alias);
        self.m_vkBuffer.set(buffer);
        self.m_vkMappedPtr.set(mapped);
    }

    /// Installs the single source staging allocation while leaving `m_pool`
    /// empty, exactly as TextureVulkan does.
    pub(crate) fn installStagingBacking(
        &mut self,
        buffer: vk::Buffer,
        allocation: vk_mem::Allocation,
        mapped: *mut u8,
    ) {
        debug_assert!(self.m_vkBuffer.get().is_null());
        debug_assert!(self.m_pool.get_mut().is_empty());
        *self.m_vmaAllocation.get_mut() = AllocationMirror::Owned(Box::new(allocation));
        self.m_vkBuffer.set(buffer);
        self.m_vkMappedPtr.set(mapped);
    }
}

impl Deref for BufferVulkan {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BufferVulkan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

unsafe impl GpuResourcePayload for BufferVulkan {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_ore_metal::gpu_resource::GPUResourceManagerOwner;

    #[test]
    fn preserves_offset_zero_buffer_base_and_source_defaults() {
        use core::mem::offset_of;

        assert_eq!(offset_of!(BufferVulkan, base), 0);
        let owner = GPUResourceManagerOwner::new();
        let buffer = BufferVulkan::new(owner.manager(), 64, BufferUsage::uniform);
        assert_eq!(buffer.size(), 64);
        assert_eq!(buffer.usage(), BufferUsage::uniform);
        assert!(buffer.current().is_null());
        assert!(unsafe { (&*buffer.m_vmaAllocation.get()).isNull() });
        assert!(buffer.m_vkMappedPtr.get().is_null());
        assert!(unsafe { (&*buffer.m_pool.get()).is_empty() });
        assert_eq!(buffer.m_currentIndex.get(), 0);
        assert!(!buffer.m_boundSinceUpdate.get());
        assert!(buffer.m_vkUsage.is_empty());
        drop(buffer);
        owner.shutdown();
    }
}
