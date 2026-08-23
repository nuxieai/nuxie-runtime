//! Complete mechanical declaration translation of
//! `renderer/src/ore/vulkan/ore_shader_module_vulkan.hpp`.

#![allow(non_snake_case)]

use ash::vk;
use nuxie_ore_metal::gpu_resource::{GPUResource, GpuResourcePayload};
use nuxie_ore_metal::shader_module::ShaderModule;
use std::ops::{Deref, DerefMut};

#[repr(C)]
pub(crate) struct ShaderModuleVulkan {
    base: ShaderModule,
    pub(crate) m_vkDevice: vk::Device,
    pub(crate) m_vkShaderModule: vk::ShaderModule,
    pub(crate) m_vkDestroyShaderModule: Option<vk::PFN_vkDestroyShaderModule>,
}

impl ShaderModuleVulkan {
    pub(crate) fn new() -> Self {
        Self {
            base: nuxie_ore_metal::new_shader_module_backend_base(),
            m_vkDevice: vk::Device::null(),
            m_vkShaderModule: vk::ShaderModule::null(),
            m_vkDestroyShaderModule: None,
        }
    }

    /// Publishes the native handle/function triple created by `ContextVulkan`.
    ///
    /// # Safety
    /// All three values must belong to the same live Vulkan device, and that
    /// device must remain live until this resource is released by its manager.
    pub(crate) unsafe fn setNativeShaderModule(
        &mut self,
        device: vk::Device,
        shader_module: vk::ShaderModule,
        destroy_shader_module: vk::PFN_vkDestroyShaderModule,
    ) {
        debug_assert_eq!(self.m_vkShaderModule, vk::ShaderModule::null());
        self.m_vkDevice = device;
        self.m_vkShaderModule = shader_module;
        self.m_vkDestroyShaderModule = Some(destroy_shader_module);
    }
}

impl Deref for ShaderModuleVulkan {
    type Target = ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for ShaderModuleVulkan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

unsafe impl GpuResourcePayload for ShaderModuleVulkan {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
}
