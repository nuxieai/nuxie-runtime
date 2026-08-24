//! Complete mechanical implementation translation of
//! `renderer/src/ore/vulkan/ore_pipeline_vulkan.cpp`.

#![allow(non_snake_case)]

use super::ore_bind_group_layout_vulkan_decl::BindGroupLayoutVulkan;
use super::ore_context_vulkan_decl::{ContextVulkan, VKRenderPassKey};
use super::ore_pipeline_vulkan_decl::PipelineVulkan;
use super::ore_shader_module_vulkan_decl::ShaderModuleVulkan;
use super::ore_vulkan_dsl::kVkMaxGroups;
use ash::vk;
use nuxie_ore_metal::bind_group_layout::{
    BindGroupLayout, validateColorRequiresFragment, validateLayoutBasesAgainstBindingMap,
};
use nuxie_ore_metal::gpu_resource::{AnyResourceHandle, ResourceHandle};
use nuxie_ore_metal::types::{
    BlendFactor, BlendOp, ColorWriteMask, CompareFunction, CullMode, FaceWinding, LoadOp,
    PipelineDesc, PrimitiveTopology, StencilFaceState, StencilOp, StoreOp, TextureFormat,
    VertexFormat, VertexStepMode,
};
use std::ffi::CString;
use std::mem::ManuallyDrop;

fn hasStencilLocal(format: TextureFormat) -> bool {
    matches!(
        format,
        TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
    )
}

fn oreVertexFormatToVk(format: VertexFormat) -> vk::Format {
    match format {
        VertexFormat::float1 => vk::Format::R32_SFLOAT,
        VertexFormat::float2 => vk::Format::R32G32_SFLOAT,
        VertexFormat::float3 => vk::Format::R32G32B32_SFLOAT,
        VertexFormat::float4 => vk::Format::R32G32B32A32_SFLOAT,
        VertexFormat::uint8x4 => vk::Format::R8G8B8A8_UINT,
        VertexFormat::sint8x4 => vk::Format::R8G8B8A8_SINT,
        VertexFormat::unorm8x4 => vk::Format::R8G8B8A8_UNORM,
        VertexFormat::snorm8x4 => vk::Format::R8G8B8A8_SNORM,
        VertexFormat::uint16x2 => vk::Format::R16G16_UINT,
        VertexFormat::sint16x2 => vk::Format::R16G16_SINT,
        VertexFormat::unorm16x2 => vk::Format::R16G16_UNORM,
        VertexFormat::snorm16x2 => vk::Format::R16G16_SNORM,
        VertexFormat::uint16x4 => vk::Format::R16G16B16A16_UINT,
        VertexFormat::sint16x4 => vk::Format::R16G16B16A16_SINT,
        VertexFormat::float16x2 => vk::Format::R16G16_SFLOAT,
        VertexFormat::float16x4 => vk::Format::R16G16B16A16_SFLOAT,
        VertexFormat::uint32 => vk::Format::R32_UINT,
    }
}

fn oreTopologyToVk(topology: PrimitiveTopology) -> vk::PrimitiveTopology {
    match topology {
        PrimitiveTopology::pointList => vk::PrimitiveTopology::POINT_LIST,
        PrimitiveTopology::lineList => vk::PrimitiveTopology::LINE_LIST,
        PrimitiveTopology::lineStrip => vk::PrimitiveTopology::LINE_STRIP,
        PrimitiveTopology::triangleList => vk::PrimitiveTopology::TRIANGLE_LIST,
        PrimitiveTopology::triangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
    }
}

fn oreCullModeToVk(mode: CullMode) -> vk::CullModeFlags {
    match mode {
        CullMode::none => vk::CullModeFlags::NONE,
        CullMode::front => vk::CullModeFlags::FRONT,
        CullMode::back => vk::CullModeFlags::BACK,
    }
}

fn oreWindingToVk(winding: FaceWinding) -> vk::FrontFace {
    // Identity mapping. Vertex Y-flip is baked into the shader by naga.
    match winding {
        FaceWinding::counterClockwise => vk::FrontFace::COUNTER_CLOCKWISE,
        FaceWinding::clockwise => vk::FrontFace::CLOCKWISE,
    }
}

fn oreBlendFactorToVk(factor: BlendFactor) -> vk::BlendFactor {
    match factor {
        BlendFactor::zero => vk::BlendFactor::ZERO,
        BlendFactor::one => vk::BlendFactor::ONE,
        BlendFactor::srcColor => vk::BlendFactor::SRC_COLOR,
        BlendFactor::oneMinusSrcColor => vk::BlendFactor::ONE_MINUS_SRC_COLOR,
        BlendFactor::srcAlpha => vk::BlendFactor::SRC_ALPHA,
        BlendFactor::oneMinusSrcAlpha => vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        BlendFactor::dstColor => vk::BlendFactor::DST_COLOR,
        BlendFactor::oneMinusDstColor => vk::BlendFactor::ONE_MINUS_DST_COLOR,
        BlendFactor::dstAlpha => vk::BlendFactor::DST_ALPHA,
        BlendFactor::oneMinusDstAlpha => vk::BlendFactor::ONE_MINUS_DST_ALPHA,
        BlendFactor::srcAlphaSaturated => vk::BlendFactor::SRC_ALPHA_SATURATE,
        BlendFactor::blendColor => vk::BlendFactor::CONSTANT_COLOR,
        BlendFactor::oneMinusBlendColor => vk::BlendFactor::ONE_MINUS_CONSTANT_COLOR,
    }
}

fn oreBlendOpToVk(op: BlendOp) -> vk::BlendOp {
    match op {
        BlendOp::add => vk::BlendOp::ADD,
        BlendOp::subtract => vk::BlendOp::SUBTRACT,
        BlendOp::reverseSubtract => vk::BlendOp::REVERSE_SUBTRACT,
        BlendOp::min => vk::BlendOp::MIN,
        BlendOp::max => vk::BlendOp::MAX,
    }
}

fn oreColorWriteMaskToVk(mask: ColorWriteMask) -> vk::ColorComponentFlags {
    let mut result = vk::ColorComponentFlags::empty();
    if (mask & ColorWriteMask::red) != ColorWriteMask::none {
        result |= vk::ColorComponentFlags::R;
    }
    if (mask & ColorWriteMask::green) != ColorWriteMask::none {
        result |= vk::ColorComponentFlags::G;
    }
    if (mask & ColorWriteMask::blue) != ColorWriteMask::none {
        result |= vk::ColorComponentFlags::B;
    }
    if (mask & ColorWriteMask::alpha) != ColorWriteMask::none {
        result |= vk::ColorComponentFlags::A;
    }
    result
}

fn oreCompareFuncToVk(function: CompareFunction) -> vk::CompareOp {
    match function {
        CompareFunction::none | CompareFunction::always => vk::CompareOp::ALWAYS,
        CompareFunction::never => vk::CompareOp::NEVER,
        CompareFunction::less => vk::CompareOp::LESS,
        CompareFunction::equal => vk::CompareOp::EQUAL,
        CompareFunction::lessEqual => vk::CompareOp::LESS_OR_EQUAL,
        CompareFunction::greater => vk::CompareOp::GREATER,
        CompareFunction::notEqual => vk::CompareOp::NOT_EQUAL,
        CompareFunction::greaterEqual => vk::CompareOp::GREATER_OR_EQUAL,
    }
}

fn oreStencilOpToVk(op: StencilOp) -> vk::StencilOp {
    match op {
        StencilOp::keep => vk::StencilOp::KEEP,
        StencilOp::zero => vk::StencilOp::ZERO,
        StencilOp::replace => vk::StencilOp::REPLACE,
        StencilOp::incrementClamp => vk::StencilOp::INCREMENT_AND_CLAMP,
        StencilOp::decrementClamp => vk::StencilOp::DECREMENT_AND_CLAMP,
        StencilOp::invert => vk::StencilOp::INVERT,
        StencilOp::incrementWrap => vk::StencilOp::INCREMENT_AND_WRAP,
        StencilOp::decrementWrap => vk::StencilOp::DECREMENT_AND_WRAP,
    }
}

fn oreStencilFaceToVk(state: &StencilFaceState, readMask: u8, writeMask: u8) -> vk::StencilOpState {
    vk::StencilOpState {
        fail_op: oreStencilOpToVk(state.failOp),
        pass_op: oreStencilOpToVk(state.passOp),
        depth_fail_op: oreStencilOpToVk(state.depthFailOp),
        compare_op: oreCompareFuncToVk(state.compare),
        compare_mask: u32::from(readMask),
        write_mask: u32::from(writeMask),
        reference: 0,
    }
}

impl Drop for PipelineVulkan {
    fn drop(&mut self) {
        unsafe {
            if self.m_vkPipeline != vk::Pipeline::null() {
                if let Some(destroy) = self.m_vkDestroyPipeline {
                    destroy(self.m_vkDevice, self.m_vkPipeline, std::ptr::null());
                }
            }
            if self.m_vkPipelineLayout != vk::PipelineLayout::null() {
                if let Some(destroy) = self.m_vkDestroyPipelineLayout {
                    destroy(self.m_vkDevice, self.m_vkPipelineLayout, std::ptr::null());
                }
            }
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl Drop for BindGroupLayoutVulkan {
    fn drop(&mut self) {
        if self.m_vkDSL != vk::DescriptorSetLayout::null() {
            if let Some(destroy) = self.m_vkDestroyDescriptorSetLayout {
                unsafe { destroy(self.m_vkDevice, self.m_vkDSL, std::ptr::null()) };
            }
        }
    }
}

pub(crate) fn makePipeline(
    context: &mut ContextVulkan,
    desc: &PipelineDesc<'_>,
    mut outError: Option<&mut String>,
) -> Option<AnyResourceHandle> {
    let manager = nuxie_ore_metal::context_backend_manager(&context.base)
        .expect("ContextVulkan requires its source GPUResourceManager");
    let mut pipeline = PipelineVulkan::new(manager.clone(), desc)?;
    pipeline.m_vkDevice = context.m_vk.device;
    pipeline.m_vkTopology = oreTopologyToVk(desc.topology);

    // Pipeline's backend-independent constructor cannot know this sibling
    // crate's concrete shader subclass, so copy the same source binding map at
    // the concrete boundary.
    let sourceModule = desc.vertexModule.or(desc.fragmentModule)?;
    let sourceModule = sourceModule.downcast_ref::<ShaderModuleVulkan>()?;
    *pipeline.base.m_bindingMap = sourceModule.m_bindingMap.clone();

    let layoutHandles = desc.bindGroupLayouts.unwrap_or(&[]);
    let layoutHandles = layoutHandles.get(..desc.bindGroupLayoutCount as usize)?;
    let layoutBases = layoutHandles
        .iter()
        .map(|layout| {
            layout.map(|layout| {
                let concrete = layout
                    .downcast_ref::<BindGroupLayoutVulkan>()
                    .expect("Vulkan pipeline requires Vulkan bind-group layouts");
                &**concrete as &BindGroupLayout
            })
        })
        .collect::<Vec<_>>();

    let mut error = String::new();
    if !validateLayoutBasesAgainstBindingMap(
        &pipeline.base.m_bindingMap,
        desc.bindGroupLayouts.map(|_| layoutBases.as_slice()),
        desc.bindGroupLayoutCount,
        Some(&mut error),
    ) || !validateColorRequiresFragment(
        desc.colorCount,
        desc.fragmentModule.is_some(),
        Some(&mut error),
    ) {
        if let Some(out) = outError.as_deref_mut() {
            *out = error;
        } else {
            context.setLastError(format!("makePipeline: {error}"));
        }
        return None;
    }

    let mut dsls = [vk::DescriptorSetLayout::null(); kVkMaxGroups as usize];
    let mut emptyDSL = vk::DescriptorSetLayout::null();
    for group in 0..desc.bindGroupLayoutCount.min(kVkMaxGroups) as usize {
        dsls[group] = if let Some(layout) = layoutHandles[group] {
            layout.downcast_ref::<BindGroupLayoutVulkan>()?.m_vkDSL
        } else {
            if emptyDSL == vk::DescriptorSetLayout::null() {
                emptyDSL = context.vkGetOrCreateEmptyDSL();
            }
            emptyDSL
        };
    }
    let setLayouts = &dsls[..desc.bindGroupLayoutCount as usize];
    let layoutInfo = vk::PipelineLayoutCreateInfo::default().set_layouts(setLayouts);
    pipeline.m_vkPipelineLayout = unsafe {
        context
            .m_vk
            .m_ashDevice
            .create_pipeline_layout(&layoutInfo, None)
    }
    .unwrap_or(vk::PipelineLayout::null());
    pipeline.m_vkDestroyPipelineLayout =
        Some(context.m_vk.m_ashDevice.fp_v1_0().destroy_pipeline_layout);

    let vertexEntry = CString::new(desc.vertexEntryPoint.unwrap_or("vs_main")).ok()?;
    let vertexModule = desc.vertexModule?.downcast_ref::<ShaderModuleVulkan>()?;
    let mut stages = Vec::with_capacity(2);
    stages.push(
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertexModule.m_vkShaderModule)
            .name(&vertexEntry),
    );
    let fragmentEntry = desc
        .fragmentModule
        .map(|_| CString::new(desc.fragmentEntryPoint.unwrap_or("fs_main")))
        .transpose()
        .ok()?;
    if let Some(fragmentHandle) = desc.fragmentModule {
        let fragmentModule = fragmentHandle.downcast_ref::<ShaderModuleVulkan>()?;
        stages.push(
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragmentModule.m_vkShaderModule)
                .name(fragmentEntry.as_ref().expect("fragment entry point")),
        );
    }

    const MAX_BINDINGS: usize = 8;
    const MAX_ATTRIBS: usize = 32;
    let vertexBuffers = desc.vertexBuffers.unwrap_or(&[]);
    let vertexBuffers = vertexBuffers.get(..desc.vertexBufferCount as usize)?;
    assert!(vertexBuffers.len() <= MAX_BINDINGS);
    let mut bindings = Vec::with_capacity(MAX_BINDINGS);
    let mut attributes = Vec::with_capacity(MAX_ATTRIBS);
    for (binding, layout) in vertexBuffers.iter().enumerate() {
        bindings.push(vk::VertexInputBindingDescription {
            binding: binding as u32,
            stride: layout.stride,
            input_rate: if layout.stepMode == VertexStepMode::instance {
                vk::VertexInputRate::INSTANCE
            } else {
                vk::VertexInputRate::VERTEX
            },
        });
        let sourceAttributes = layout
            .attributes
            .get(..layout.attributeCount as usize)
            .expect("vertex attributeCount exceeds its authored span");
        for attribute in sourceAttributes {
            assert!(attributes.len() < MAX_ATTRIBS);
            attributes.push(vk::VertexInputAttributeDescription {
                location: attribute.shaderSlot,
                binding: binding as u32,
                format: oreVertexFormatToVk(attribute.format),
                offset: attribute.offset,
            });
        }
    }
    let vertexInput = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&bindings)
        .vertex_attribute_descriptions(&attributes);
    let inputAssembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(pipeline.m_vkTopology)
        .primitive_restart_enable(false);
    let viewportState = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    let mut raster = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(oreCullModeToVk(desc.cullMode))
        .front_face(oreWindingToVk(desc.winding))
        .line_width(1.0);
    if desc.depthStencil.depthBias != 0 || desc.depthStencil.depthBiasSlopeScale != 0.0 {
        raster = raster
            .depth_bias_enable(true)
            .depth_bias_constant_factor(desc.depthStencil.depthBias as f32)
            .depth_bias_slope_factor(desc.depthStencil.depthBiasSlopeScale)
            .depth_bias_clamp(desc.depthStencil.depthBiasClamp);
    }
    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::from_raw(desc.sampleCount));

    let hasDepthStencil = desc.depthStencil.format != TextureFormat::rgba8unorm;
    let stencilTestEnabled = hasDepthStencil && hasStencilLocal(desc.depthStencil.format);
    let depthStencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(
            hasDepthStencil && desc.depthStencil.depthCompare != CompareFunction::always,
        )
        .depth_write_enable(hasDepthStencil && desc.depthStencil.depthWriteEnabled)
        .depth_compare_op(oreCompareFuncToVk(desc.depthStencil.depthCompare))
        .stencil_test_enable(stencilTestEnabled)
        .front(oreStencilFaceToVk(
            &desc.stencilFront,
            desc.stencilReadMask,
            desc.stencilWriteMask,
        ))
        .back(oreStencilFaceToVk(
            &desc.stencilBack,
            desc.stencilReadMask,
            desc.stencilWriteMask,
        ));
    pipeline.m_vkStencilTestEnabled = stencilTestEnabled;

    assert!(desc.colorCount <= 4);
    let mut blendAttachments = Vec::with_capacity(4);
    for target in &desc.colorTargets[..desc.colorCount as usize] {
        let mut attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(oreColorWriteMaskToVk(target.writeMask))
            .blend_enable(target.blendEnabled);
        if target.blendEnabled {
            attachment = attachment
                .src_color_blend_factor(oreBlendFactorToVk(target.blend.srcColor))
                .dst_color_blend_factor(oreBlendFactorToVk(target.blend.dstColor))
                .color_blend_op(oreBlendOpToVk(target.blend.colorOp))
                .src_alpha_blend_factor(oreBlendFactorToVk(target.blend.srcAlpha))
                .dst_alpha_blend_factor(oreBlendFactorToVk(target.blend.dstAlpha))
                .alpha_blend_op(oreBlendOpToVk(target.blend.alphaOp));
        }
        blendAttachments.push(attachment);
    }
    let colorBlend =
        vk::PipelineColorBlendStateCreateInfo::default().attachments(&blendAttachments);
    let dynamicStates = [
        vk::DynamicState::VIEWPORT,
        vk::DynamicState::SCISSOR,
        vk::DynamicState::STENCIL_REFERENCE,
        vk::DynamicState::BLEND_CONSTANTS,
    ];
    let dynamicState = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamicStates);

    let mut key = VKRenderPassKey {
        colorCount: desc.colorCount,
        sampleCount: desc.sampleCount,
        ..Default::default()
    };
    for index in 0..desc.colorCount as usize {
        key.colorFormats[index] = desc.colorTargets[index].format;
        key.colorLoadOps[index] = LoadOp::dontCare;
        key.colorStoreOps[index] = StoreOp::discard;
    }
    if hasDepthStencil {
        key.depthFormat = desc.depthStencil.format;
        key.depthLoadOp = LoadOp::dontCare;
        key.depthStoreOp = StoreOp::discard;
        key.hasDepth = true;
    }
    let compatRenderPass = context.getOrCreateRenderPass(&key);

    let mut pipelineInfo = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertexInput)
        .input_assembly_state(&inputAssembly)
        .viewport_state(&viewportState)
        .rasterization_state(&raster)
        .multisample_state(&multisample)
        .depth_stencil_state(&depthStencil)
        .dynamic_state(&dynamicState)
        .layout(pipeline.m_vkPipelineLayout)
        .render_pass(compatRenderPass)
        .subpass(0);
    if desc.fragmentModule.is_some() {
        pipelineInfo = pipelineInfo.color_blend_state(&colorBlend);
    }
    match unsafe {
        context.m_vk.m_ashDevice.create_graphics_pipelines(
            vk::PipelineCache::null(),
            std::slice::from_ref(&pipelineInfo),
            None,
        )
    } {
        Ok(created) => pipeline.m_vkPipeline = created[0],
        Err((_, result)) => {
            context.setLastError(format!(
                "Ore Vulkan: vkCreateGraphicsPipelines failed ({})",
                result.as_raw()
            ));
            if let Some(out) = outError.as_deref_mut() {
                *out = context.lastError();
            }
            return None;
        }
    }
    pipeline.m_vkDestroyPipeline = Some(context.m_vk.m_ashDevice.fp_v1_0().destroy_pipeline);
    Some(ResourceHandle::new(Some(manager), pipeline).erase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_tables_match_the_source_contract() {
        assert_eq!(
            oreVertexFormatToVk(VertexFormat::uint32),
            vk::Format::R32_UINT
        );
        assert_eq!(
            oreTopologyToVk(PrimitiveTopology::lineStrip),
            vk::PrimitiveTopology::LINE_STRIP
        );
        assert_eq!(
            oreCompareFuncToVk(CompareFunction::none),
            vk::CompareOp::ALWAYS
        );
        assert_eq!(
            oreStencilOpToVk(StencilOp::incrementWrap),
            vk::StencilOp::INCREMENT_AND_WRAP
        );
        assert_eq!(
            oreColorWriteMaskToVk(ColorWriteMask::red | ColorWriteMask::alpha),
            vk::ColorComponentFlags::R | vk::ColorComponentFlags::A
        );
    }
}
