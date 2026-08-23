//! Complete mechanical declaration translation of
//! `renderer/src/vulkan/render_pass_vulkan.hpp`.

#![allow(non_snake_case, non_upper_case_globals)]

use super::draw_pipeline_layout_vulkan_decl::DrawPipelineLayoutVulkan;
use super::vulkan_context_decl::VulkanContext;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    InterlockMode, LoadAction,
};
use ash::vk;
use std::ops::{BitAnd, BitOr, BitOrAssign, Not};
use std::sync::Arc;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderPassOptionsVulkan(pub(crate) u32);

impl RenderPassOptionsVulkan {
    pub(crate) const none: Self = Self(0);
    pub(crate) const fixedFunctionColorOutput: Self = Self(1 << 0);
    pub(crate) const manuallyResolved: Self = Self(1 << 1);
    pub(crate) const rasterOrderingInterruptible: Self = Self(1 << 2);
    pub(crate) const rasterOrderingResume: Self = Self(1 << 3);
    pub(crate) const atomicCoalescedResolveAndTransfer: Self = Self(1 << 4);
    pub(crate) const msaaSeedFromOffscreenTexture: Self = Self(1 << 5);

    pub(crate) const fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}

impl BitOr for RenderPassOptionsVulkan {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl BitOrAssign for RenderPassOptionsVulkan {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
impl BitAnd for RenderPassOptionsVulkan {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
impl Not for RenderPassOptionsVulkan {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

pub(crate) const RENDER_PASS_OPTION_COUNT: u64 = 6;
pub(crate) const RENDER_PASS_OPTIONS_LAYOUT_MASK: RenderPassOptionsVulkan = RenderPassOptionsVulkan(
    !(RenderPassOptionsVulkan::rasterOrderingInterruptible.0
        | RenderPassOptionsVulkan::rasterOrderingResume.0),
);

pub(crate) const FORMAT_BIT_COUNT: u64 = 9;
pub(crate) const LOAD_OP_BIT_COUNT: u64 = 2;
pub(crate) const KEY_NO_INTERLOCK_MODE_BIT_COUNT: u64 =
    FORMAT_BIT_COUNT + RENDER_PASS_OPTION_COUNT + LOAD_OP_BIT_COUNT;
pub(crate) const KEY_BIT_COUNT: u64 = KEY_NO_INTERLOCK_MODE_BIT_COUNT + 3;
const _: () = assert!(KEY_BIT_COUNT <= 32);

pub(crate) struct RenderPassVulkan {
    pub(crate) m_vk: Arc<VulkanContext>,
    pub(crate) m_drawPipelineLayout: *const DrawPipelineLayoutVulkan,
    pub(crate) m_renderPass: vk::RenderPass,
}

impl RenderPassVulkan {
    pub(crate) fn KeyNoInterlockMode(
        renderPassOptions: RenderPassOptionsVulkan,
        renderTargetFormat: vk::Format,
        loadAction: LoadAction,
    ) -> u32 {
        super::render_pass_vulkan_impl::KeyNoInterlockMode(
            renderPassOptions,
            renderTargetFormat,
            loadAction,
        )
    }

    pub(crate) fn Key(
        interlockMode: InterlockMode,
        renderPassOptions: RenderPassOptionsVulkan,
        renderTargetFormat: vk::Format,
        loadAction: LoadAction,
    ) -> u32 {
        super::render_pass_vulkan_impl::Key(
            interlockMode,
            renderPassOptions,
            renderTargetFormat,
            loadAction,
        )
    }

    pub(crate) fn drawPipelineLayout(&self) -> Option<&DrawPipelineLayoutVulkan> {
        unsafe { self.m_drawPipelineLayout.as_ref() }
    }
}
