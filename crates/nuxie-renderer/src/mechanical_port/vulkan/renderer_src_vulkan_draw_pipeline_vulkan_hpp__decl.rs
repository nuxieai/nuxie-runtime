//! Complete mechanical declaration translation of
//! `renderer/src/vulkan/draw_pipeline_vulkan.hpp`.

#![allow(non_snake_case)]

use super::render_pass_vulkan_decl::RenderPassOptionsVulkan;
use super::vulkan_context_decl::VulkanContext;
#[cfg(feature = "with-rive-tools")]
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    DrawContents, DrawType, InterlockMode, LoadAction, PlatformFeatures, ShaderFeatures,
    ShaderMiscFlags,
};
use ash::vk;
use nuxie_render_api::BlendMode;
use std::sync::Arc;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DrawPipelineOptions(pub(crate) u32);

impl DrawPipelineOptions {
    pub(crate) const none: Self = Self(0);
    pub(crate) const wireframe: Self = Self(1 << 0);

    pub(crate) const fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}

pub(crate) const DRAW_PIPELINE_OPTION_COUNT: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PipelineProps {
    pub(crate) drawType: DrawType,
    pub(crate) shaderFeatures: ShaderFeatures,
    pub(crate) interlockMode: InterlockMode,
    pub(crate) shaderMiscFlags: ShaderMiscFlags,
    pub(crate) drawContents: DrawContents,
    pub(crate) blendMode: BlendMode,
    pub(crate) drawPipelineOptions: DrawPipelineOptions,
    pub(crate) renderPassOptions: RenderPassOptionsVulkan,
    pub(crate) renderTargetFormat: vk::Format,
    pub(crate) colorLoadAction: LoadAction,
    #[cfg(feature = "with-rive-tools")]
    pub(crate) synthesizedFailureType: SynthesizedFailureType,
}

impl PipelineProps {
    pub(crate) fn createKey(&self, platformFeatures: &PlatformFeatures) -> u64 {
        super::draw_pipeline_vulkan_impl::createKey(self, platformFeatures)
    }
}

pub(crate) struct DrawPipelineVulkan {
    pub(crate) m_vk: Arc<VulkanContext>,
    pub(crate) m_vkPipeline: vk::Pipeline,
}

impl From<&DrawPipelineVulkan> for vk::Pipeline {
    fn from(value: &DrawPipelineVulkan) -> Self {
        value.m_vkPipeline
    }
}
