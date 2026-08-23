//! Complete mechanical declaration translation of
//! `renderer/src/ore/vulkan/ore_sampler_vulkan.hpp`.

#![allow(non_snake_case)]

use ash::vk;
use nuxie_ore_metal::gpu_resource::{GPUResource, GpuResourcePayload};
use nuxie_ore_metal::sampler::Sampler;
use std::ops::{Deref, DerefMut};

#[repr(C)]
pub(crate) struct SamplerVulkan {
    base: Sampler,
    pub(crate) m_vkDevice: vk::Device,
    pub(crate) m_vkSampler: vk::Sampler,
    pub(crate) m_vkDestroySampler: Option<vk::PFN_vkDestroySampler>,
}

impl SamplerVulkan {
    pub(crate) fn new() -> Self {
        Self {
            base: nuxie_ore_metal::new_sampler_backend_base(),
            m_vkDevice: vk::Device::null(),
            m_vkSampler: vk::Sampler::null(),
            m_vkDestroySampler: None,
        }
    }

    /// Publishes the native handle/function triple created by `ContextVulkan`.
    ///
    /// # Safety
    /// All three values must belong to the same live Vulkan device, and that
    /// device must remain live until this resource is released by its manager.
    pub(crate) unsafe fn setNativeSampler(
        &mut self,
        device: vk::Device,
        sampler: vk::Sampler,
        destroy_sampler: vk::PFN_vkDestroySampler,
    ) {
        debug_assert_eq!(self.m_vkSampler, vk::Sampler::null());
        self.m_vkDevice = device;
        self.m_vkSampler = sampler;
        self.m_vkDestroySampler = Some(destroy_sampler);
    }
}

impl Deref for SamplerVulkan {
    type Target = Sampler;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for SamplerVulkan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

unsafe impl GpuResourcePayload for SamplerVulkan {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
}
