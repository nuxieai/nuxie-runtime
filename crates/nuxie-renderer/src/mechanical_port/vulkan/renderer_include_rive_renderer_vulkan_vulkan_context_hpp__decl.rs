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
    pub(crate) GetDeviceProcAddr: Option<vk::PFN_vkGetDeviceProcAddr>,
    pub(crate) GetPhysicalDeviceFormatProperties: Option<vk::PFN_vkGetPhysicalDeviceFormatProperties>,
    pub(crate) GetPhysicalDeviceProperties: Option<vk::PFN_vkGetPhysicalDeviceProperties>,
    pub(crate) GetPhysicalDeviceFeatures: Option<vk::PFN_vkGetPhysicalDeviceFeatures>,
    pub(crate) SetDebugUtilsObjectNameEXT: Option<vk::PFN_vkSetDebugUtilsObjectNameEXT>,
    pub(crate) AllocateCommandBuffers: Option<vk::PFN_vkAllocateCommandBuffers>,
    pub(crate) AllocateDescriptorSets: Option<vk::PFN_vkAllocateDescriptorSets>,
    pub(crate) BeginCommandBuffer: Option<vk::PFN_vkBeginCommandBuffer>,
    pub(crate) CmdBeginRenderPass: Option<vk::PFN_vkCmdBeginRenderPass>,
    pub(crate) CmdBindDescriptorSets: Option<vk::PFN_vkCmdBindDescriptorSets>,
    pub(crate) CmdBindIndexBuffer: Option<vk::PFN_vkCmdBindIndexBuffer>,
    pub(crate) CmdBindPipeline: Option<vk::PFN_vkCmdBindPipeline>,
    pub(crate) CmdBindVertexBuffers: Option<vk::PFN_vkCmdBindVertexBuffers>,
    pub(crate) CmdBlitImage: Option<vk::PFN_vkCmdBlitImage>,
    pub(crate) CmdClearColorImage: Option<vk::PFN_vkCmdClearColorImage>,
    pub(crate) CmdCopyBufferToImage: Option<vk::PFN_vkCmdCopyBufferToImage>,
    pub(crate) CmdDraw: Option<vk::PFN_vkCmdDraw>,
    pub(crate) CmdDrawIndexed: Option<vk::PFN_vkCmdDrawIndexed>,
    pub(crate) CmdEndRenderPass: Option<vk::PFN_vkCmdEndRenderPass>,
    pub(crate) CmdFillBuffer: Option<vk::PFN_vkCmdFillBuffer>,
    pub(crate) CmdNextSubpass: Option<vk::PFN_vkCmdNextSubpass>,
    pub(crate) CmdPipelineBarrier: Option<vk::PFN_vkCmdPipelineBarrier>,
    pub(crate) CmdSetBlendConstants: Option<vk::PFN_vkCmdSetBlendConstants>,
    pub(crate) CmdSetColorWriteEnableEXT: Option<vk::PFN_vkCmdSetColorWriteEnableEXT>,
    pub(crate) CmdSetCullMode: Option<vk::PFN_vkCmdSetCullMode>,
    pub(crate) CmdSetDepthWriteEnable: Option<vk::PFN_vkCmdSetDepthWriteEnable>,
    pub(crate) CmdSetScissor: Option<vk::PFN_vkCmdSetScissor>,
    pub(crate) CmdSetStencilCompareMask: Option<vk::PFN_vkCmdSetStencilCompareMask>,
    pub(crate) CmdSetStencilOp: Option<vk::PFN_vkCmdSetStencilOp>,
    pub(crate) CmdSetStencilReference: Option<vk::PFN_vkCmdSetStencilReference>,
    pub(crate) CmdSetStencilWriteMask: Option<vk::PFN_vkCmdSetStencilWriteMask>,
    pub(crate) CmdSetViewport: Option<vk::PFN_vkCmdSetViewport>,
    pub(crate) CreateCommandPool: Option<vk::PFN_vkCreateCommandPool>,
    pub(crate) CreateDescriptorPool: Option<vk::PFN_vkCreateDescriptorPool>,
    pub(crate) CreateDescriptorSetLayout: Option<vk::PFN_vkCreateDescriptorSetLayout>,
    pub(crate) CreateFramebuffer: Option<vk::PFN_vkCreateFramebuffer>,
    pub(crate) CreateFence: Option<vk::PFN_vkCreateFence>,
    pub(crate) CreateGraphicsPipelines: Option<vk::PFN_vkCreateGraphicsPipelines>,
    pub(crate) CreateImageView: Option<vk::PFN_vkCreateImageView>,
    pub(crate) CreatePipelineLayout: Option<vk::PFN_vkCreatePipelineLayout>,
    pub(crate) CreateRenderPass: Option<vk::PFN_vkCreateRenderPass>,
    pub(crate) CreateSampler: Option<vk::PFN_vkCreateSampler>,
    pub(crate) CreateShaderModule: Option<vk::PFN_vkCreateShaderModule>,
    pub(crate) DestroyCommandPool: Option<vk::PFN_vkDestroyCommandPool>,
    pub(crate) DestroyDescriptorPool: Option<vk::PFN_vkDestroyDescriptorPool>,
    pub(crate) DestroyDescriptorSetLayout: Option<vk::PFN_vkDestroyDescriptorSetLayout>,
    pub(crate) DestroyFence: Option<vk::PFN_vkDestroyFence>,
    pub(crate) DestroyFramebuffer: Option<vk::PFN_vkDestroyFramebuffer>,
    pub(crate) DestroyImageView: Option<vk::PFN_vkDestroyImageView>,
    pub(crate) DestroyPipeline: Option<vk::PFN_vkDestroyPipeline>,
    pub(crate) DestroyPipelineLayout: Option<vk::PFN_vkDestroyPipelineLayout>,
    pub(crate) DestroyRenderPass: Option<vk::PFN_vkDestroyRenderPass>,
    pub(crate) DestroySampler: Option<vk::PFN_vkDestroySampler>,
    pub(crate) DestroyShaderModule: Option<vk::PFN_vkDestroyShaderModule>,
    pub(crate) EndCommandBuffer: Option<vk::PFN_vkEndCommandBuffer>,
    pub(crate) FreeCommandBuffers: Option<vk::PFN_vkFreeCommandBuffers>,
    pub(crate) FreeDescriptorSets: Option<vk::PFN_vkFreeDescriptorSets>,
    pub(crate) QueueSubmit: Option<vk::PFN_vkQueueSubmit>,
    pub(crate) QueueWaitIdle: Option<vk::PFN_vkQueueWaitIdle>,
    pub(crate) ResetCommandBuffer: Option<vk::PFN_vkResetCommandBuffer>,
    pub(crate) ResetDescriptorPool: Option<vk::PFN_vkResetDescriptorPool>,
    pub(crate) ResetFences: Option<vk::PFN_vkResetFences>,
    pub(crate) UpdateDescriptorSets: Option<vk::PFN_vkUpdateDescriptorSets>,
    pub(crate) WaitForFences: Option<vk::PFN_vkWaitForFences>,
    // Ash loaders remain execution helpers; the source-published command
    // fields above retain the exact per-command identity and order.
    pub(crate) m_ashInstance: ash::Instance,
    pub(crate) m_ashDevice: ash::Device,
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
