//! Complete mechanical declaration translation of
//! `renderer/src/ore/vulkan/ore_bind_group_layout_vulkan.hpp`.
//!
//! The pinned source declares its destructor here but defines it in
//! `ore_pipeline_vulkan.cpp`; the matching Rust `Drop` implementation remains
//! owned by that later translation target as well.

#![allow(non_snake_case)]

use ash::vk;
use nuxie_ore_metal::bind_group_layout::BindGroupLayout;
use nuxie_ore_metal::gpu_resource::{GPUResource, GpuResourcePayload};
use std::ops::{Deref, DerefMut};

#[repr(C)]
pub(crate) struct BindGroupLayoutVulkan {
    base: BindGroupLayout,
    pub(crate) m_vkDevice: vk::Device,
    pub(crate) m_vkDSL: vk::DescriptorSetLayout,
    pub(crate) m_vkDestroyDescriptorSetLayout: Option<vk::PFN_vkDestroyDescriptorSetLayout>,
}

impl BindGroupLayoutVulkan {
    pub(crate) fn new() -> Self {
        Self {
            base: nuxie_ore_metal::new_bind_group_layout_backend_base(),
            m_vkDevice: vk::Device::null(),
            m_vkDSL: vk::DescriptorSetLayout::null(),
            m_vkDestroyDescriptorSetLayout: None,
        }
    }

    /// Publishes the native layout/function triple created by `ContextVulkan`.
    ///
    /// # Safety
    /// All three values must belong to the same live Vulkan device, and that
    /// device must remain live until this resource is released by its manager.
    pub(crate) unsafe fn setNativeDescriptorSetLayout(
        &mut self,
        device: vk::Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
        destroy_descriptor_set_layout: vk::PFN_vkDestroyDescriptorSetLayout,
    ) {
        debug_assert_eq!(self.m_vkDSL, vk::DescriptorSetLayout::null());
        self.m_vkDevice = device;
        self.m_vkDSL = descriptor_set_layout;
        self.m_vkDestroyDescriptorSetLayout = Some(destroy_descriptor_set_layout);
    }
}

impl Deref for BindGroupLayoutVulkan {
    type Target = BindGroupLayout;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BindGroupLayoutVulkan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

unsafe impl GpuResourcePayload for BindGroupLayoutVulkan {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }

    fn bind_group_layout_base(&self) -> Option<&BindGroupLayout> {
        Some(&self.base)
    }
}
