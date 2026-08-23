//! Complete mechanical declaration translation of
//! `renderer/src/vulkan/draw_shader_vulkan.hpp`.

#![allow(non_snake_case)]

use super::vulkan_context_decl::VulkanContext;
use ash::vk;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DrawShaderVulkanType {
    vertex,
    fragment,
}

/// One retained Vulkan context and the uniquely owned shader module created by
/// the pinned constructor. Source copy construction and assignment are absent.
pub(crate) struct DrawShaderVulkan {
    pub(super) m_vk: Arc<VulkanContext>,
    pub(super) m_module: vk::ShaderModule,
}

impl DrawShaderVulkan {
    pub(crate) fn module(&self) -> vk::ShaderModule {
        self.m_module
    }
}
