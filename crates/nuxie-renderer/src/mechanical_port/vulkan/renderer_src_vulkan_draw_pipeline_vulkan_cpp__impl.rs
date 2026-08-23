//! Complete mechanical implementation translation of
//! `renderer/src/vulkan/draw_pipeline_vulkan.cpp`.

#![allow(non_snake_case)]

use super::common_layouts_decl as layout;
use super::draw_pipeline_layout_vulkan_decl::DrawPipelineLayoutVulkan;
use super::draw_pipeline_vulkan_decl::{
    DrawPipelineVulkan, PipelineProps, DRAW_PIPELINE_OPTION_COUNT,
};
use super::draw_shader_vulkan_decl::DrawShaderVulkan;
use super::render_pass_vulkan_decl::{
    RenderPassOptionsVulkan, RenderPassVulkan, KEY_NO_INTERLOCK_MODE_BIT_COUNT,
};
use super::vkutil_decl;
use super::vulkan_context_decl::VulkanContext;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    BlendEquation, DrawType, InterlockMode, LoadAction, PlatformFeatures, ShaderFeatures,
    ShaderMiscFlags, DEPTH_MAX, DEPTH_MIN,
};
use crate::mechanical_port::source::renderer::src::gpu_cpp::{
    get_pipeline_state, pipeline_unique_key,
};
use ash::vk;
use std::sync::Arc;

const COLOR_PLANE_IDX: usize = 0;
const CLIP_PLANE_IDX: usize = 1;
const PLS_PLANE_COUNT: usize = 4;
const SPECIALIZATION_COUNT: usize = 14;

#[cold]
#[track_caller]
fn source_unreachable() -> ! {
    panic!("RIVE_UNREACHABLE in pinned draw_pipeline_vulkan.cpp")
}

const fn vk_blend_op(equation: BlendEquation) -> vk::BlendOp {
    match equation {
        BlendEquation::none | BlendEquation::srcOver | BlendEquation::plus => vk::BlendOp::ADD,
        BlendEquation::min => vk::BlendOp::MIN,
        BlendEquation::max => vk::BlendOp::MAX,
        BlendEquation::screen
        | BlendEquation::overlay
        | BlendEquation::darken
        | BlendEquation::lighten
        | BlendEquation::colorDodge
        | BlendEquation::colorBurn
        | BlendEquation::hardLight
        | BlendEquation::softLight
        | BlendEquation::difference
        | BlendEquation::exclusion
        | BlendEquation::multiply
        | BlendEquation::hue
        | BlendEquation::saturation
        | BlendEquation::color
        | BlendEquation::luminosity => panic!("RIVE_UNREACHABLE"),
    }
}

const fn vk_dst_blend_factor(equation: BlendEquation) -> vk::BlendFactor {
    match equation {
        BlendEquation::none => vk::BlendFactor::ZERO,
        BlendEquation::srcOver => vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        BlendEquation::plus | BlendEquation::min | BlendEquation::max => vk::BlendFactor::ONE,
        BlendEquation::screen
        | BlendEquation::overlay
        | BlendEquation::darken
        | BlendEquation::lighten
        | BlendEquation::colorDodge
        | BlendEquation::colorBurn
        | BlendEquation::hardLight
        | BlendEquation::softLight
        | BlendEquation::difference
        | BlendEquation::exclusion
        | BlendEquation::multiply
        | BlendEquation::hue
        | BlendEquation::saturation
        | BlendEquation::color
        | BlendEquation::luminosity => panic!("RIVE_UNREACHABLE"),
    }
}

#[inline]
const fn add_bits_to_key(key: u64, value: u64, bitCount: u64) -> u64 {
    assert!(bitCount < 64);
    assert!(value < (1 << bitCount));
    assert!(key <= (u64::MAX >> bitCount));
    (key << bitCount) | value
}

pub(crate) fn createKey(props: &PipelineProps, platformFeatures: &PlatformFeatures) -> u64 {
    let mut key = pipeline_unique_key(
        props.drawType,
        props.shaderFeatures,
        props.interlockMode,
        props.shaderMiscFlags,
        props.drawContents,
        props
            .renderPassOptions
            .has(RenderPassOptionsVulkan::fixedFunctionColorOutput),
        props.blendMode,
        platformFeatures,
    );
    let renderPassKeyNoInterlockMode = RenderPassVulkan::KeyNoInterlockMode(
        props.renderPassOptions,
        props.renderTargetFormat,
        props.colorLoadAction,
    );
    key = add_bits_to_key(
        key,
        renderPassKeyNoInterlockMode as u64,
        KEY_NO_INTERLOCK_MODE_BIT_COUNT,
    );
    key = add_bits_to_key(
        key,
        props.drawPipelineOptions.0 as u64,
        DRAW_PIPELINE_OPTION_COUNT as u64,
    );
    add_bits_to_key(
        key,
        u64::from(vkutil_decl::hasPipelineDynamicState(props.drawType)),
        1,
    )
}

pub(crate) fn subpass_index(
    drawType: DrawType,
    colorLoadAction: LoadAction,
    interlockMode: InterlockMode,
    shaderMiscFlags: ShaderMiscFlags,
) -> u32 {
    if interlockMode == InterlockMode::clockwiseAtomic {
        return if shaderMiscFlags.has(ShaderMiscFlags::borrowedCoveragePass) {
            0
        } else {
            1
        };
    }
    let mainSubpassIdx = u32::from(
        interlockMode == InterlockMode::msaa && colorLoadAction == LoadAction::preserveRenderTarget,
    );
    match drawType {
        DrawType::renderPassInitialize => {
            assert_eq!(mainSubpassIdx, 1);
            0
        }
        DrawType::midpointFanPatches
        | DrawType::midpointFanCenterAAPatches
        | DrawType::outerCurvePatches
        | DrawType::interiorTriangulation
        | DrawType::featherAtlasBlit
        | DrawType::imageRect
        | DrawType::imageMesh
        | DrawType::msaaStrokes
        | DrawType::msaaMidpointFanBorrowedCoverage
        | DrawType::msaaDynamicMidpointFans
        | DrawType::msaaMidpointFans
        | DrawType::msaaMidpointFanStencilReset
        | DrawType::msaaMidpointFanPathsStencil
        | DrawType::msaaMidpointFanPathsCover
        | DrawType::msaaOuterCubics
        | DrawType::clipReset => mainSubpassIdx,
        DrawType::renderPassResolve => mainSubpassIdx + 1,
    }
}

impl DrawPipelineVulkan {
    /// The manager-owned shader lookups and vendor ID are passed after the
    /// exact synchronous cache operations. This keeps construction source-
    /// ordered while representing the C++ manager cycle without a fallback.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        vk: &Arc<VulkanContext>,
        pipelineLayout: &DrawPipelineLayoutVulkan,
        props: &PipelineProps,
        vkRenderPass: vk::RenderPass,
        platformFeatures: &PlatformFeatures,
        vertShader: &DrawShaderVulkan,
        fragShader: &DrawShaderVulkan,
        vendorID: u32,
    ) -> Self {
        #[cfg(feature = "with-rive-tools")]
        if props.synthesizedFailureType
            == crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType::pipelineCreation
        {
            return Self {
                m_vk: Arc::clone(vk),
                m_vkPipeline: vk::Pipeline::null(),
            };
        }
        if vertShader.module() == vk::ShaderModule::null()
            || fragShader.module() == vk::ShaderModule::null()
        {
            return Self {
                m_vk: Arc::clone(vk),
                m_vkPipeline: vk::Pipeline::null(),
            };
        }

        let pipelineWriteOnlyRenderTarget = props
            .renderPassOptions
            .has(RenderPassOptionsVulkan::fixedFunctionColorOutput);
        let pipelineState = get_pipeline_state(
            props.drawType,
            props.interlockMode,
            props.shaderMiscFlags,
            props.drawContents,
            pipelineWriteOnlyRenderTarget,
            props.blendMode,
            platformFeatures,
        );
        let interlockMode = pipelineLayout.interlockMode();
        let subpassIndex = subpass_index(
            props.drawType,
            props.colorLoadAction,
            interlockMode,
            props.shaderMiscFlags,
        );

        let shaderPermutationFlags: [u32; SPECIALIZATION_COUNT] = [
            u32::from(props.shaderFeatures.0 & ShaderFeatures::ENABLE_CLIPPING.0 != 0),
            u32::from(props.shaderFeatures.0 & ShaderFeatures::ENABLE_CLIP_RECT.0 != 0),
            u32::from(props.shaderFeatures.0 & ShaderFeatures::ENABLE_ADVANCED_BLEND.0 != 0),
            u32::from(props.shaderFeatures.0 & ShaderFeatures::ENABLE_FEATHER.0 != 0),
            u32::from(props.shaderFeatures.0 & ShaderFeatures::ENABLE_EVEN_ODD.0 != 0),
            u32::from(props.shaderFeatures.0 & ShaderFeatures::ENABLE_NESTED_CLIPPING.0 != 0),
            u32::from(props.shaderFeatures.0 & ShaderFeatures::ENABLE_HSL_BLEND_MODES.0 != 0),
            u32::from(props.shaderFeatures.0 & ShaderFeatures::ENABLE_DITHER.0 != 0),
            u32::from(props.shaderMiscFlags.has(ShaderMiscFlags::clockwiseFill)),
            u32::from(
                props
                    .shaderMiscFlags
                    .has(ShaderMiscFlags::nestedClipUpdateOnly),
            ),
            u32::from(
                props
                    .shaderMiscFlags
                    .has(ShaderMiscFlags::borrowedCoveragePass),
            ),
            u32::from(props.shaderMiscFlags.has(ShaderMiscFlags::storeColorClear)),
            u32::from(
                props
                    .shaderMiscFlags
                    .has(ShaderMiscFlags::loadColorFromDstTexture),
            ),
            u32::from(vendorID == vkutil_decl::ARM),
        ];
        let permutationMapEntries: [vk::SpecializationMapEntry; SPECIALIZATION_COUNT] =
            std::array::from_fn(|i| vk::SpecializationMapEntry {
                constant_id: i as u32,
                offset: (i * std::mem::size_of::<u32>()) as u32,
                size: std::mem::size_of::<u32>(),
            });
        let specializationInfo = vk::SpecializationInfo {
            map_entry_count: SPECIALIZATION_COUNT as u32,
            p_map_entries: permutationMapEntries.as_ptr(),
            data_size: std::mem::size_of_val(&shaderPermutationFlags),
            p_data: shaderPermutationFlags.as_ptr().cast(),
            ..Default::default()
        };
        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertShader.module())
                .name(c"main")
                .specialization_info(&specializationInfo),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragShader.module())
                .name(c"main")
                .specialization_info(&specializationInfo),
        ];
        let pipelineRasterizationStateCreateInfo =
            vk::PipelineRasterizationStateCreateInfo::default()
                .polygon_mode(
                    if props
                        .drawPipelineOptions
                        .has(super::draw_pipeline_vulkan_decl::DrawPipelineOptions::wireframe)
                    {
                        vk::PolygonMode::LINE
                    } else {
                        vk::PolygonMode::FILL
                    },
                )
                .cull_mode(vkutil_decl::vkCullMode(pipelineState.cullFace))
                .front_face(vk::FrontFace::CLOCKWISE)
                .line_width(1.0);

        let mut blendEquation = pipelineState.blendEquation;
        let mut colorWriteEnabled = pipelineState.colorWriteEnabled;
        if interlockMode == InterlockMode::rasterOrdering
            || interlockMode == InterlockMode::atomics
            || (interlockMode == InterlockMode::clockwiseAtomic
                && !props
                    .shaderMiscFlags
                    .has(ShaderMiscFlags::borrowedCoveragePass))
        {
            colorWriteEnabled = true;
        }
        if interlockMode == InterlockMode::atomics
            && !props
                .shaderMiscFlags
                .has(ShaderMiscFlags::coalescedResolveAndTransfer)
        {
            blendEquation = BlendEquation::srcOver;
        } else if props
            .shaderMiscFlags
            .has(ShaderMiscFlags::coalescedResolveAndTransfer)
        {
            debug_assert_eq!(interlockMode, InterlockMode::atomics);
            debug_assert_eq!(blendEquation, BlendEquation::none);
        }

        let attachmentCount = pipelineLayout
            .colorAttachmentCount(subpassIndex, pipelineLayout.renderPassOptions())
            as usize;
        assert!(attachmentCount <= PLS_PLANE_COUNT);
        let blendState = vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(blendEquation != BlendEquation::none)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk_dst_blend_factor(blendEquation))
            .color_blend_op(vk_blend_op(blendEquation))
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk_dst_blend_factor(blendEquation))
            .alpha_blend_op(vk_blend_op(blendEquation))
            .color_write_mask(if colorWriteEnabled {
                vkutil_decl::kColorWriteMaskRGBA
            } else {
                vkutil_decl::kColorWriteMaskNone
            });
        let mut blendStates = vec![blendState; attachmentCount];
        if vk.features.independentBlend && interlockMode == InterlockMode::clockwiseAtomic {
            if props.shaderMiscFlags.has(ShaderMiscFlags::clipUpdateOnly) {
                blendStates[COLOR_PLANE_IDX].color_write_mask = vkutil_decl::kColorWriteMaskNone;
            } else if props.drawType != DrawType::renderPassInitialize {
                assert_ne!(props.drawType, DrawType::clipReset);
                assert!(!props
                    .shaderMiscFlags
                    .has(ShaderMiscFlags::nestedClipUpdateOnly));
                blendStates[CLIP_PLANE_IDX].color_write_mask = vkutil_decl::kColorWriteMaskNone;
            }
        }
        let mut pipelineColorBlendStateCreateInfo =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(&blendStates);
        if interlockMode == InterlockMode::rasterOrdering
            && vk.features.rasterizationOrderColorAttachmentAccess
        {
            pipelineColorBlendStateCreateInfo.flags |=
                vk::PipelineColorBlendStateCreateFlags::RASTERIZATION_ORDER_ATTACHMENT_ACCESS_EXT;
        }

        let mut depthStencilState = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(pipelineState.depthTestEnabled)
            .depth_write_enable(pipelineState.depthWriteEnabled)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(pipelineState.stencilTestEnabled)
            .min_depth_bounds(DEPTH_MIN)
            .max_depth_bounds(DEPTH_MAX);
        if pipelineState.stencilTestEnabled {
            let front = vk::StencilOpState {
                fail_op: vkutil_decl::vkStencilOp(pipelineState.stencilFrontOps.stencilFailOp),
                pass_op: vkutil_decl::vkStencilOp(pipelineState.stencilFrontOps.depthStencilPassOp),
                depth_fail_op: vkutil_decl::vkStencilOp(pipelineState.stencilFrontOps.depthFailOp),
                compare_op: vkutil_decl::vkCompareOp(pipelineState.stencilFrontOps.compareOp),
                compare_mask: pipelineState.stencilCompareMask as u32,
                write_mask: pipelineState.stencilWriteMask as u32,
                reference: pipelineState.stencilReference as u32,
            };
            let back = if !pipelineState.stencilDoubleSided {
                front
            } else {
                vk::StencilOpState {
                    fail_op: vkutil_decl::vkStencilOp(pipelineState.stencilBackOps.stencilFailOp),
                    pass_op: vkutil_decl::vkStencilOp(
                        pipelineState.stencilBackOps.depthStencilPassOp,
                    ),
                    depth_fail_op: vkutil_decl::vkStencilOp(
                        pipelineState.stencilBackOps.depthFailOp,
                    ),
                    compare_op: vkutil_decl::vkCompareOp(pipelineState.stencilBackOps.compareOp),
                    compare_mask: pipelineState.stencilCompareMask as u32,
                    write_mask: pipelineState.stencilWriteMask as u32,
                    reference: pipelineState.stencilReference as u32,
                }
            };
            depthStencilState.front = front;
            depthStencilState.back = back;
        }
        let msaaState = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(
            if interlockMode == InterlockMode::msaa && props.drawType != DrawType::renderPassResolve
            {
                vk::SampleCountFlags::TYPE_4
            } else {
                vk::SampleCountFlags::TYPE_1
            },
        );

        let (vertexInputState, inputAssemblyState) = match props.drawType {
            DrawType::midpointFanPatches
            | DrawType::midpointFanCenterAAPatches
            | DrawType::outerCurvePatches
            | DrawType::msaaOuterCubics
            | DrawType::msaaStrokes
            | DrawType::msaaMidpointFanBorrowedCoverage
            | DrawType::msaaDynamicMidpointFans
            | DrawType::msaaMidpointFans
            | DrawType::msaaMidpointFanStencilReset
            | DrawType::msaaMidpointFanPathsStencil
            | DrawType::msaaMidpointFanPathsCover => (
                &*layout::PATH_VERTEX_INPUT_STATE,
                &*layout::INPUT_ASSEMBLY_TRIANGLE_LIST,
            ),
            DrawType::clipReset | DrawType::interiorTriangulation | DrawType::featherAtlasBlit => (
                &*layout::INTERIOR_TRI_VERTEX_INPUT_STATE,
                &*layout::INPUT_ASSEMBLY_TRIANGLE_LIST,
            ),
            DrawType::imageRect => (
                &*layout::IMAGE_RECT_VERTEX_INPUT_STATE,
                &*layout::INPUT_ASSEMBLY_TRIANGLE_LIST,
            ),
            DrawType::imageMesh => (
                &*layout::IMAGE_MESH_VERTEX_INPUT_STATE,
                &*layout::INPUT_ASSEMBLY_TRIANGLE_LIST,
            ),
            DrawType::renderPassResolve | DrawType::renderPassInitialize => (
                &*layout::EMPTY_VERTEX_INPUT_STATE,
                &*layout::INPUT_ASSEMBLY_TRIANGLE_STRIP,
            ),
        };
        let dynamicState = if vkutil_decl::hasPipelineDynamicState(props.drawType) {
            &*layout::DYNAMIC_PIPELINE_STATE
        } else {
            &*layout::DYNAMIC_VIEWPORT_SCISSOR
        };
        let pipelineCreateInfo = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(vertexInputState)
            .input_assembly_state(inputAssemblyState)
            .viewport_state(&layout::SINGLE_VIEWPORT)
            .rasterization_state(&pipelineRasterizationStateCreateInfo)
            .multisample_state(&msaaState)
            .depth_stencil_state(&depthStencilState)
            .color_blend_state(&pipelineColorBlendStateCreateInfo)
            .dynamic_state(dynamicState)
            .layout(pipelineLayout.vkPipelineLayout())
            .render_pass(vkRenderPass)
            .subpass(subpassIndex);
        let vkPipeline = match unsafe {
            vk.m_ashDevice.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[pipelineCreateInfo],
                None,
            )
        } {
            Ok(mut pipelines) => pipelines.remove(0),
            Err(_) => vk::Pipeline::null(),
        };
        Self {
            m_vk: Arc::clone(vk),
            m_vkPipeline: vkPipeline,
        }
    }
}

impl Drop for DrawPipelineVulkan {
    fn drop(&mut self) {
        unsafe {
            self.m_vk
                .m_ashDevice
                .destroy_pipeline(self.m_vkPipeline, None)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw_pipeline_vulkan_decl::DrawPipelineOptions;
    use super::*;
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::DrawContents;
    use nuxie_render_api::BlendMode;

    fn props(drawType: DrawType, interlockMode: InterlockMode) -> PipelineProps {
        PipelineProps {
            drawType,
            shaderFeatures: ShaderFeatures::NONE,
            interlockMode,
            shaderMiscFlags: ShaderMiscFlags::none,
            drawContents: DrawContents::none,
            blendMode: BlendMode::SrcOver,
            drawPipelineOptions: DrawPipelineOptions::none,
            renderPassOptions: RenderPassOptionsVulkan::none,
            renderTargetFormat: vk::Format::R8G8B8A8_UNORM,
            colorLoadAction: LoadAction::clear,
            #[cfg(feature = "with-rive-tools")]
            synthesizedFailureType: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType::none,
        }
    }

    #[test]
    fn source_subpass_matrix_is_preserved() {
        assert_eq!(
            subpass_index(
                DrawType::midpointFanPatches,
                LoadAction::clear,
                InterlockMode::rasterOrdering,
                ShaderMiscFlags::none,
            ),
            0
        );
        assert_eq!(
            subpass_index(
                DrawType::renderPassInitialize,
                LoadAction::preserveRenderTarget,
                InterlockMode::msaa,
                ShaderMiscFlags::none,
            ),
            0
        );
        assert_eq!(
            subpass_index(
                DrawType::renderPassResolve,
                LoadAction::preserveRenderTarget,
                InterlockMode::msaa,
                ShaderMiscFlags::none,
            ),
            2
        );
        assert_eq!(
            subpass_index(
                DrawType::midpointFanPatches,
                LoadAction::clear,
                InterlockMode::clockwiseAtomic,
                ShaderMiscFlags::borrowedCoveragePass,
            ),
            0
        );
        assert_eq!(
            subpass_index(
                DrawType::midpointFanPatches,
                LoadAction::clear,
                InterlockMode::clockwiseAtomic,
                ShaderMiscFlags::none,
            ),
            1
        );
    }

    #[test]
    fn dynamic_state_choice_has_a_distinct_pipeline_key() {
        let features = PlatformFeatures::default();
        let dynamic = props(DrawType::msaaDynamicMidpointFans, InterlockMode::msaa);
        let mut baked = dynamic;
        baked.drawType = DrawType::msaaMidpointFans;
        assert_ne!(createKey(&dynamic, &features), createKey(&baked, &features));
    }

    #[test]
    fn source_blend_conversion_only_accepts_fixed_function_equations() {
        assert_eq!(vk_blend_op(BlendEquation::srcOver), vk::BlendOp::ADD);
        assert_eq!(
            vk_dst_blend_factor(BlendEquation::none),
            vk::BlendFactor::ZERO
        );
        assert_eq!(
            vk_dst_blend_factor(BlendEquation::srcOver),
            vk::BlendFactor::ONE_MINUS_SRC_ALPHA
        );
    }
}
