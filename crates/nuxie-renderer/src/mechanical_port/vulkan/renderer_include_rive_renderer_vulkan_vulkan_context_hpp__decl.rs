//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/vulkan/vulkan_context.hpp`.

#![allow(non_snake_case, non_upper_case_globals)]

use ash::vk;
use nuxie_ore_metal::gpu_resource::{GPUResourceManager, GPUResourceManagerOwner};
use std::mem::ManuallyDrop;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VulkanFeatures {
    pub(crate) apiVersion: u32,
    pub(crate) independentBlend: bool,
    pub(crate) fillModeNonSolid: bool,
    pub(crate) fragmentStoresAndAtomics: bool,
    pub(crate) shaderClipDistance: bool,
    pub(crate) rasterizationOrderColorAttachmentAccess: bool,
    pub(crate) fragmentShaderPixelInterlock: bool,
    pub(crate) colorWriteEnable: bool,
    pub(crate) VK_KHR_portability_subset: bool,
    pub(crate) textureCompressionBC: bool,
    pub(crate) textureCompressionASTC_LDR: bool,
    pub(crate) textureCompressionETC2: bool,
}

impl Default for VulkanFeatures {
    fn default() -> Self {
        Self {
            apiVersion: vk::API_VERSION_1_1,
            independentBlend: false,
            fillModeNonSolid: false,
            fragmentStoresAndAtomics: false,
            shaderClipDistance: false,
            rasterizationOrderColorAttachmentAccess: false,
            fragmentShaderPixelInterlock: false,
            colorWriteEnable: false,
            VK_KHR_portability_subset: false,
            textureCompressionBC: false,
            textureCompressionASTC_LDR: false,
            textureCompressionETC2: false,
        }
    }
}

pub(crate) const RIVE_VULKAN_INSTANCE_COMMANDS: [&str; 5] = [
    "GetDeviceProcAddr",
    "GetPhysicalDeviceFormatProperties",
    "GetPhysicalDeviceProperties",
    "GetPhysicalDeviceFeatures",
    "SetDebugUtilsObjectNameEXT",
];

pub(crate) const RIVE_VULKAN_DEVICE_COMMANDS: [&str; 59] = [
    "AllocateCommandBuffers", "AllocateDescriptorSets", "BeginCommandBuffer",
    "CmdBeginRenderPass", "CmdBindDescriptorSets", "CmdBindIndexBuffer",
    "CmdBindPipeline", "CmdBindVertexBuffers", "CmdBlitImage", "CmdClearColorImage",
    "CmdCopyBufferToImage", "CmdDraw", "CmdDrawIndexed", "CmdEndRenderPass",
    "CmdFillBuffer", "CmdNextSubpass", "CmdPipelineBarrier", "CmdSetBlendConstants",
    "CmdSetColorWriteEnableEXT", "CmdSetCullMode", "CmdSetDepthWriteEnable",
    "CmdSetScissor", "CmdSetStencilCompareMask", "CmdSetStencilOp",
    "CmdSetStencilReference", "CmdSetStencilWriteMask", "CmdSetViewport",
    "CreateCommandPool", "CreateDescriptorPool", "CreateDescriptorSetLayout",
    "CreateFramebuffer", "CreateFence", "CreateGraphicsPipelines", "CreateImageView",
    "CreatePipelineLayout", "CreateRenderPass", "CreateSampler", "CreateShaderModule",
    "DestroyCommandPool", "DestroyDescriptorPool", "DestroyDescriptorSetLayout",
    "DestroyFence", "DestroyFramebuffer", "DestroyImageView", "DestroyPipeline",
    "DestroyPipelineLayout", "DestroyRenderPass", "DestroySampler", "DestroyShaderModule",
    "EndCommandBuffer", "FreeCommandBuffers", "FreeDescriptorSets", "QueueSubmit",
    "QueueWaitIdle", "ResetCommandBuffer", "ResetDescriptorPool", "ResetFences",
    "UpdateDescriptorSets", "WaitForFences",
];

pub(crate) struct VulkanContext {
    pub(crate) instance: vk::Instance,
    pub(crate) physicalDevice: vk::PhysicalDevice,
    pub(crate) device: vk::Device,
    pub(crate) features: VulkanFeatures,
    pub(crate) m_ashInstance: ash::Instance,
    pub(crate) m_ashDevice: ash::Device,
    pub(crate) m_setDebugUtilsObjectNameEXT: Option<vk::PFN_vkSetDebugUtilsObjectNameEXT>,
    pub(crate) CmdSetColorWriteEnableEXT: Option<vk::PFN_vkCmdSetColorWriteEnableEXT>,
    pub(crate) CmdSetCullMode: Option<vk::PFN_vkCmdSetCullMode>,
    pub(crate) CmdSetDepthWriteEnable: Option<vk::PFN_vkCmdSetDepthWriteEnable>,
    pub(crate) m_vmaAllocator: ManuallyDrop<vk_mem::Allocator>,
    pub(crate) m_physicalDeviceProperties: vk::PhysicalDeviceProperties,
    pub(crate) m_supportsD24S8: bool,
    // Source inheritance destroys the GPUResourceManager base last.
    pub(crate) m_managerOwner: GPUResourceManagerOwner,
}

impl VulkanContext {
    pub(crate) fn manager(&self) -> GPUResourceManager { self.m_managerOwner.manager() }
    pub(crate) fn allocator(&self) -> &vk_mem::Allocator { &self.m_vmaAllocator }
    pub(crate) fn physicalDeviceProperties(&self) -> &vk::PhysicalDeviceProperties {
        &self.m_physicalDeviceProperties
    }
    pub(crate) fn supportsD24S8(&self) -> bool { self.m_supportsD24S8 }
    pub(crate) fn ashDevice(&self) -> &ash::Device { &self.m_ashDevice }
    pub(crate) fn ashInstance(&self) -> &ash::Instance { &self.m_ashInstance }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_source_command_denominators_are_frozen() {
        assert_eq!(RIVE_VULKAN_INSTANCE_COMMANDS, [
            "GetDeviceProcAddr", "GetPhysicalDeviceFormatProperties",
            "GetPhysicalDeviceProperties", "GetPhysicalDeviceFeatures",
            "SetDebugUtilsObjectNameEXT",
        ]);
        assert_eq!(RIVE_VULKAN_DEVICE_COMMANDS.len(), 59);
        assert_eq!(RIVE_VULKAN_DEVICE_COMMANDS.first(), Some(&"AllocateCommandBuffers"));
        assert_eq!(RIVE_VULKAN_DEVICE_COMMANDS.last(), Some(&"WaitForFences"));
    }

    #[test]
    fn source_feature_defaults_are_vulkan_1_1_and_disabled() {
        let features = VulkanFeatures::default();
        assert_eq!(features.apiVersion, vk::API_VERSION_1_1);
        assert!(!features.independentBlend);
        assert!(!features.fillModeNonSolid);
        assert!(!features.fragmentStoresAndAtomics);
        assert!(!features.shaderClipDistance);
        assert!(!features.rasterizationOrderColorAttachmentAccess);
        assert!(!features.fragmentShaderPixelInterlock);
        assert!(!features.colorWriteEnable);
        assert!(!features.VK_KHR_portability_subset);
        assert!(!features.textureCompressionBC);
        assert!(!features.textureCompressionASTC_LDR);
        assert!(!features.textureCompressionETC2);
    }
}
