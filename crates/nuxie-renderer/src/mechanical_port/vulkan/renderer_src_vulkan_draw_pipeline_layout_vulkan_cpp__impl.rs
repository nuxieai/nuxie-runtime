//! Complete mechanical implementation translation of
//! `renderer/src/vulkan/draw_pipeline_layout_vulkan.cpp`.

#![allow(non_snake_case)]

use super::draw_pipeline_layout_vulkan_decl::DrawPipelineLayoutVulkan;
use super::render_pass_vulkan_decl::RenderPassOptionsVulkan;
use super::vulkan_context_decl::VulkanContext;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::InterlockMode;
use ash::vk;
use std::sync::Arc;

const COLOR_PLANE_IDX: u32 = 0;
const CLIP_PLANE_IDX: u32 = 1;
const SCRATCH_COLOR_PLANE_IDX: u32 = 2;
const COVERAGE_PLANE_IDX: u32 = 3;
const PLS_PLANE_COUNT: usize = 4;
const VULKAN_BINDINGS_SET_COUNT: usize = 3;

#[cold]
#[track_caller]
fn vk_check<T>(result: Result<T, vk::Result>) -> T {
    match result {
        Ok(value) => value,
        Err(result) => super::vkutil_impl::vk_abort(result, file!(), line!()),
    }
}

impl DrawPipelineLayoutVulkan {
    /// `storageTexturePLS` is the exact value returned by
    /// `PipelineManagerVulkan::plsBackingType(interlockMode)`. Passing the
    /// source-owned value explicitly breaks the C++ declaration cycle without
    /// changing its choice or lifetime.
    pub(crate) fn new(
        vk: &Arc<VulkanContext>,
        interlockMode: InterlockMode,
        renderPassOptions: RenderPassOptionsVulkan,
        storageTexturePLS: bool,
        perFlushDescriptorSetLayout: vk::DescriptorSetLayout,
        perDrawDescriptorSetLayout: vk::DescriptorSetLayout,
    ) -> Self {
        let fixedFunctionColorOutput =
            renderPassOptions.has(RenderPassOptionsVulkan::fixedFunctionColorOutput);
        let plsDescriptorType = if storageTexturePLS {
            vk::DescriptorType::STORAGE_IMAGE
        } else {
            vk::DescriptorType::INPUT_ATTACHMENT
        };

        let mut plsLayoutBindings = Vec::with_capacity(PLS_PLANE_COUNT);
        if !fixedFunctionColorOutput {
            plsLayoutBindings.push(
                vk::DescriptorSetLayoutBinding::default()
                    .binding(COLOR_PLANE_IDX)
                    .descriptor_type(plsDescriptorType)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            );
        }
        if interlockMode != InterlockMode::msaa {
            plsLayoutBindings.push(
                vk::DescriptorSetLayoutBinding::default()
                    .binding(CLIP_PLANE_IDX)
                    .descriptor_type(plsDescriptorType)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            );
        }
        if interlockMode == InterlockMode::rasterOrdering
            || ((interlockMode == InterlockMode::clockwise
                || interlockMode == InterlockMode::clockwiseAtomic)
                && !fixedFunctionColorOutput)
        {
            plsLayoutBindings.push(
                vk::DescriptorSetLayoutBinding::default()
                    .binding(SCRATCH_COLOR_PLANE_IDX)
                    .descriptor_type(if interlockMode == InterlockMode::clockwiseAtomic {
                        vk::DescriptorType::STORAGE_IMAGE
                    } else {
                        plsDescriptorType
                    })
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            );
        }
        if interlockMode == InterlockMode::rasterOrdering
            || interlockMode == InterlockMode::atomics
            || interlockMode == InterlockMode::clockwise
        {
            plsLayoutBindings.push(
                vk::DescriptorSetLayoutBinding::default()
                    .binding(COVERAGE_PLANE_IDX)
                    .descriptor_type(if interlockMode == InterlockMode::atomics {
                        vk::DescriptorType::STORAGE_IMAGE
                    } else {
                        plsDescriptorType
                    })
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            );
        } else if interlockMode == InterlockMode::msaa {
            plsLayoutBindings.push(
                vk::DescriptorSetLayoutBinding::default()
                    .binding(COVERAGE_PLANE_IDX)
                    .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            );
        }

        let plsTextureDescriptorSetLayout = if plsLayoutBindings.is_empty() {
            vk::DescriptorSetLayout::null()
        } else {
            let info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&plsLayoutBindings);
            vk_check(unsafe { vk.m_ashDevice.create_descriptor_set_layout(&info, None) })
        };

        let descriptorSetLayouts = [
            perFlushDescriptorSetLayout,
            perDrawDescriptorSetLayout,
            plsTextureDescriptorSetLayout,
        ];
        const _: () = assert!(COLOR_PLANE_IDX == 0);
        const _: () = assert!(CLIP_PLANE_IDX == 1);
        const _: () = assert!(SCRATCH_COLOR_PLANE_IDX == 2);
        const _: () = assert!(COVERAGE_PLANE_IDX == 3);
        const _: () = assert!(VULKAN_BINDINGS_SET_COUNT == 3);
        let setCount = if plsTextureDescriptorSetLayout == vk::DescriptorSetLayout::null() {
            VULKAN_BINDINGS_SET_COUNT - 1
        } else {
            VULKAN_BINDINGS_SET_COUNT
        };
        let info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&descriptorSetLayouts[..setCount]);
        let pipelineLayout =
            vk_check(unsafe { vk.m_ashDevice.create_pipeline_layout(&info, None) });

        Self {
            m_vk: Arc::clone(vk),
            m_interlockMode: interlockMode,
            m_renderPassOptions: renderPassOptions,
            m_plsTextureDescriptorSetLayout: plsTextureDescriptorSetLayout,
            m_pipelineLayout: pipelineLayout,
        }
    }
}

impl Drop for DrawPipelineLayoutVulkan {
    fn drop(&mut self) {
        unsafe {
            self.m_vk
                .m_ashDevice
                .destroy_descriptor_set_layout(self.m_plsTextureDescriptorSetLayout, None);
            self.m_vk
                .m_ashDevice
                .destroy_pipeline_layout(self.m_pipelineLayout, None);
        }
    }
}

pub(crate) fn colorAttachmentCount(
    layout: &DrawPipelineLayoutVulkan,
    subpassIndex: u32,
    renderPassOptions: RenderPassOptionsVulkan,
) -> u32 {
    colorAttachmentCountForMode(layout.m_interlockMode, subpassIndex, renderPassOptions)
}

fn colorAttachmentCountForMode(
    interlockMode: InterlockMode,
    subpassIndex: u32,
    renderPassOptions: RenderPassOptionsVulkan,
) -> u32 {
    match interlockMode {
        InterlockMode::rasterOrdering => {
            assert!(subpassIndex == 0 || subpassIndex == 1);
            if subpassIndex == 0 {
                4
            } else {
                1
            }
        }
        InterlockMode::atomics => {
            assert!(subpassIndex <= 1);
            2 - subpassIndex
        }
        InterlockMode::clockwise => {
            assert_eq!(subpassIndex, 0);
            u32::from(renderPassOptions.has(RenderPassOptionsVulkan::fixedFunctionColorOutput))
        }
        InterlockMode::clockwiseAtomic => {
            assert!(subpassIndex == 0 || subpassIndex == 1);
            2
        }
        InterlockMode::msaa => {
            assert!(subpassIndex <= 2);
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_attachment_count_matrix_is_preserved_without_a_live_device() {
        assert_eq!(
            colorAttachmentCountForMode(
                InterlockMode::rasterOrdering,
                0,
                RenderPassOptionsVulkan::none
            ),
            4
        );
        assert_eq!(
            colorAttachmentCountForMode(
                InterlockMode::rasterOrdering,
                1,
                RenderPassOptionsVulkan::none
            ),
            1
        );
        assert_eq!(
            colorAttachmentCountForMode(InterlockMode::atomics, 0, RenderPassOptionsVulkan::none),
            2
        );
        assert_eq!(
            colorAttachmentCountForMode(InterlockMode::atomics, 1, RenderPassOptionsVulkan::none),
            1
        );
        assert_eq!(
            colorAttachmentCountForMode(InterlockMode::clockwise, 0, RenderPassOptionsVulkan::none),
            0
        );
        assert_eq!(
            colorAttachmentCountForMode(
                InterlockMode::clockwise,
                0,
                RenderPassOptionsVulkan::fixedFunctionColorOutput,
            ),
            1
        );
        assert_eq!(
            colorAttachmentCountForMode(
                InterlockMode::clockwiseAtomic,
                0,
                RenderPassOptionsVulkan::none
            ),
            2
        );
        assert_eq!(
            colorAttachmentCountForMode(
                InterlockMode::clockwiseAtomic,
                1,
                RenderPassOptionsVulkan::none
            ),
            2
        );
        assert_eq!(
            colorAttachmentCountForMode(InterlockMode::msaa, 0, RenderPassOptionsVulkan::none),
            1
        );
        assert_eq!(
            colorAttachmentCountForMode(InterlockMode::msaa, 2, RenderPassOptionsVulkan::none),
            1
        );
    }
}
