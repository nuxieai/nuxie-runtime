//! Complete mechanical declaration translation of
//! `renderer/src/ore/vulkan/ore_pipeline_vulkan.hpp`.

#![allow(non_snake_case)]

use ash::vk;
use nuxie_ore_metal::gpu_resource::{GPUResource, GPUResourceManager, GpuResourcePayload};
use nuxie_ore_metal::pipeline::Pipeline;
use nuxie_ore_metal::types::PipelineDesc;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

#[repr(C)]
pub(crate) struct PipelineVulkan {
    pub(crate) base: ManuallyDrop<Pipeline>,
    pub(crate) m_vkDevice: vk::Device,
    pub(crate) m_vkPipeline: vk::Pipeline,
    pub(crate) m_vkPipelineLayout: vk::PipelineLayout,
    pub(crate) m_vkTopology: vk::PrimitiveTopology,
    pub(crate) m_vkStencilTestEnabled: bool,
    pub(crate) m_vkDestroyPipeline: Option<vk::PFN_vkDestroyPipeline>,
    pub(crate) m_vkDestroyPipelineLayout: Option<vk::PFN_vkDestroyPipelineLayout>,
}

impl PipelineVulkan {
    pub(crate) fn new(manager: GPUResourceManager, desc: &PipelineDesc<'_>) -> Option<Self> {
        Some(Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_pipeline_backend_base(manager, desc)?),
            m_vkDevice: vk::Device::null(),
            m_vkPipeline: vk::Pipeline::null(),
            m_vkPipelineLayout: vk::PipelineLayout::null(),
            m_vkTopology: vk::PrimitiveTopology::TRIANGLE_LIST,
            m_vkStencilTestEnabled: false,
            m_vkDestroyPipeline: None,
            m_vkDestroyPipelineLayout: None,
        })
    }
}

impl Deref for PipelineVulkan {
    type Target = Pipeline;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for PipelineVulkan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

unsafe impl GpuResourcePayload for PipelineVulkan {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
    fn pipeline_base(&self) -> Option<&Pipeline> {
        Some(&self.base)
    }
}

// Source GPU resources may be reclaimed on the manager's purgatory thread;
// recording-thread access remains enforced by ResourceHandle.
unsafe impl Send for PipelineVulkan {}
