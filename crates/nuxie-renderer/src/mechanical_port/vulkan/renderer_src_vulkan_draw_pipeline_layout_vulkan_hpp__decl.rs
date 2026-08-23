//! Complete mechanical declaration translation of
//! `renderer/src/vulkan/draw_pipeline_layout_vulkan.hpp`.

#![allow(non_snake_case)]

use super::render_pass_vulkan_decl::RenderPassOptionsVulkan;
use super::vulkan_context_decl::VulkanContext;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::InterlockMode;
use ash::vk;
use std::sync::Arc;

pub(crate) struct DrawPipelineLayoutVulkan {
    pub(crate) m_vk: Arc<VulkanContext>,
    pub(crate) m_interlockMode: InterlockMode,
    pub(crate) m_renderPassOptions: RenderPassOptionsVulkan,
    pub(crate) m_plsTextureDescriptorSetLayout: vk::DescriptorSetLayout,
    pub(crate) m_pipelineLayout: vk::PipelineLayout,
}

impl DrawPipelineLayoutVulkan {
    pub(crate) fn interlockMode(&self) -> InterlockMode {
        self.m_interlockMode
    }

    pub(crate) fn renderPassOptions(&self) -> RenderPassOptionsVulkan {
        self.m_renderPassOptions
    }

    pub(crate) fn plsLayout(&self) -> vk::DescriptorSetLayout {
        self.m_plsTextureDescriptorSetLayout
    }

    pub(crate) fn vkPipelineLayout(&self) -> vk::PipelineLayout {
        self.m_pipelineLayout
    }

    pub(crate) fn colorAttachmentCount(
        &self,
        subpassIndex: u32,
        renderPassOptions: RenderPassOptionsVulkan,
    ) -> u32 {
        super::draw_pipeline_layout_vulkan_impl::colorAttachmentCount(
            self,
            subpassIndex,
            renderPassOptions,
        )
    }
}

impl From<&DrawPipelineLayoutVulkan> for vk::PipelineLayout {
    fn from(value: &DrawPipelineLayoutVulkan) -> Self {
        value.m_pipelineLayout
    }
}
