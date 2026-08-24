//! Complete mechanical declaration translation of
//! `renderer/src/ore/vulkan/ore_bind_group_vulkan.hpp`.

#![allow(non_snake_case)]

use super::ore_buffer_vulkan_decl::BufferVulkan;
use super::ore_context_vulkan_decl::{
    ContextVulkan, ContextVulkanLifetime, DescriptorPoolGeneration,
};
use ash::vk;
use nuxie_ore_metal::bind_group::BindGroup;
use nuxie_ore_metal::gpu_resource::{GPUResource, GPUResourceManager, GpuResourcePayload};
use std::cell::{Cell, UnsafeCell};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

pub(crate) struct UBOWrite {
    pub(crate) buffer: *const BufferVulkan,
    pub(crate) dstBinding: u32,
    pub(crate) offset: u32,
    pub(crate) range: u32,
    pub(crate) r#type: vk::DescriptorType,
}

pub(crate) struct ImageWrite {
    pub(crate) dstBinding: u32,
    pub(crate) r#type: vk::DescriptorType,
    pub(crate) imageView: vk::ImageView,
    pub(crate) imageLayout: vk::ImageLayout,
    pub(crate) sampler: vk::Sampler,
}

pub(crate) struct CachedSet {
    pub(crate) key: Vec<vk::Buffer>,
    pub(crate) set: vk::DescriptorSet,
    pub(crate) pool: Option<Arc<DescriptorPoolGeneration>>,
}

/// Source-derived bind group with replayable descriptor writes and one cached
/// set for each distinct combination of uniform-buffer backings.
#[repr(C)]
pub(crate) struct BindGroupVulkan {
    pub(crate) base: ManuallyDrop<BindGroup>,
    // The backend-independent Rust base intentionally omits its unusable raw
    // Context pointer. Preserve that exact non-owning relationship here.
    pub(super) m_context: Cell<*mut ContextVulkan>,
    pub(super) m_contextLifetime: Weak<ContextVulkanLifetime>,
    pub(crate) m_uboWrites: ManuallyDrop<Vec<UBOWrite>>,
    pub(crate) m_imageWrites: ManuallyDrop<Vec<ImageWrite>>,
    pub(crate) m_vkDSL: vk::DescriptorSetLayout,
    pub(crate) m_setCache: ManuallyDrop<UnsafeCell<Vec<CachedSet>>>,
}

impl BindGroupVulkan {
    pub(crate) fn new(manager: GPUResourceManager, context: &mut ContextVulkan) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_bind_group_backend_base(manager)),
            m_context: Cell::new(context),
            m_contextLifetime: Rc::downgrade(&context.m_lifetime),
            m_uboWrites: ManuallyDrop::new(Vec::new()),
            m_imageWrites: ManuallyDrop::new(Vec::new()),
            m_vkDSL: vk::DescriptorSetLayout::null(),
            m_setCache: ManuallyDrop::new(UnsafeCell::new(Vec::new())),
        }
    }

    pub(super) fn context(&self) -> &ContextVulkan {
        assert!(
            self.m_contextLifetime
                .upgrade()
                .is_some_and(|lifetime| lifetime.isLive()),
            "BindGroupVulkan cannot access a retired ContextVulkan"
        );
        let context = self.m_context.get();
        assert!(
            !context.is_null(),
            "BindGroupVulkan requires its source ContextVulkan back-pointer"
        );
        // SAFETY: ContextVulkan owns the manager that keeps this bind group
        // alive and clears its resources before context destruction.
        unsafe { &*context }
    }

    pub(super) fn context_mut(&self) -> &mut ContextVulkan {
        assert!(
            self.m_contextLifetime
                .upgrade()
                .is_some_and(|lifetime| lifetime.isLive()),
            "BindGroupVulkan cannot access a retired ContextVulkan"
        );
        let context = self.m_context.get();
        assert!(
            !context.is_null(),
            "BindGroupVulkan requires its source ContextVulkan back-pointer"
        );
        // SAFETY: all ORE Vulkan mutation is serialized through the uniquely
        // owned ContextVulkan, matching the source immediate-mode contract.
        unsafe { &mut *context }
    }
}

impl Deref for BindGroupVulkan {
    type Target = BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BindGroupVulkan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

unsafe impl GpuResourcePayload for BindGroupVulkan {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
}

// Source GPUResource payloads may cross the manager's purgatory thread only
// for refcount/final destruction; recording-thread access remains enforced by
// ResourceHandle.
impl Drop for BindGroupVulkan {
    fn drop(&mut self) {
        // C++ destroys derived members in reverse declaration order, then the
        // BindGroup base. Descriptor pool generations therefore outlive every
        // cached set until the cache itself is released.
        unsafe {
            ManuallyDrop::drop(&mut self.m_setCache);
            ManuallyDrop::drop(&mut self.m_imageWrites);
            ManuallyDrop::drop(&mut self.m_uboWrites);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}
