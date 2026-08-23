//! Complete mechanical implementation translation of
//! `renderer/src/ore/gl/ore_render_pass_gl.cpp` for `ORE_BACKEND_GL`.

#![allow(non_snake_case)]

use super::gles3_decl::*;
use super::ore_bind_group_gl_decl::BindGroupGL;
use super::ore_buffer_gl_decl::BufferGL;
use super::ore_pipeline_gl_decl::PipelineGL;
use super::ore_render_pass_gl_decl::RenderPassGLState;
use nuxie_ore_metal::gpu_resource::AnyResourceHandle;
use nuxie_ore_metal::types::{
    BlendFactor, BlendOp, ColorWriteMask, CompareFunction, IndexFormat, PrimitiveTopology,
    StencilOp, VertexFormat, VertexStepMode,
};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_gl_ore_render_pass_gl.cpp");

pub(crate) fn oreTopologyToGL(topo: PrimitiveTopology) -> GLenum {
    match topo {
        PrimitiveTopology::pointList => GL_POINTS,
        PrimitiveTopology::lineList => GL_LINES,
        PrimitiveTopology::lineStrip => GL_LINE_STRIP,
        PrimitiveTopology::triangleList => GL_TRIANGLES,
        PrimitiveTopology::triangleStrip => GL_TRIANGLE_STRIP,
    }
}

pub(crate) fn oreBlendFactorToGL(factor: BlendFactor) -> GLenum {
    match factor {
        BlendFactor::zero => GL_ZERO,
        BlendFactor::one => GL_ONE,
        BlendFactor::srcColor => GL_SRC_COLOR,
        BlendFactor::oneMinusSrcColor => GL_ONE_MINUS_SRC_COLOR,
        BlendFactor::srcAlpha => GL_SRC_ALPHA,
        BlendFactor::oneMinusSrcAlpha => GL_ONE_MINUS_SRC_ALPHA,
        BlendFactor::dstColor => GL_DST_COLOR,
        BlendFactor::oneMinusDstColor => GL_ONE_MINUS_DST_COLOR,
        BlendFactor::dstAlpha => GL_DST_ALPHA,
        BlendFactor::oneMinusDstAlpha => GL_ONE_MINUS_DST_ALPHA,
        BlendFactor::srcAlphaSaturated => GL_SRC_ALPHA_SATURATE,
        BlendFactor::blendColor => GL_CONSTANT_COLOR,
        BlendFactor::oneMinusBlendColor => GL_ONE_MINUS_CONSTANT_COLOR,
    }
}

pub(crate) fn oreBlendOpToGL(op: BlendOp) -> GLenum {
    match op {
        BlendOp::add => GL_FUNC_ADD,
        BlendOp::subtract => GL_FUNC_SUBTRACT,
        BlendOp::reverseSubtract => GL_FUNC_REVERSE_SUBTRACT,
        BlendOp::min => GL_MIN,
        BlendOp::max => GL_MAX,
    }
}

pub(crate) fn oreCompareFunctionToGL(function: CompareFunction) -> GLenum {
    match function {
        CompareFunction::none | CompareFunction::never => GL_NEVER,
        CompareFunction::less => GL_LESS,
        CompareFunction::equal => GL_EQUAL,
        CompareFunction::lessEqual => GL_LEQUAL,
        CompareFunction::greater => GL_GREATER,
        CompareFunction::notEqual => GL_NOTEQUAL,
        CompareFunction::greaterEqual => GL_GEQUAL,
        CompareFunction::always => GL_ALWAYS,
    }
}

pub(crate) fn oreStencilOpToGL(op: StencilOp) -> GLenum {
    match op {
        StencilOp::keep => GL_KEEP,
        StencilOp::zero => GL_ZERO,
        StencilOp::replace => GL_REPLACE,
        StencilOp::incrementClamp => GL_INCR,
        StencilOp::decrementClamp => GL_DECR,
        StencilOp::invert => GL_INVERT,
        StencilOp::incrementWrap => GL_INCR_WRAP,
        StencilOp::decrementWrap => GL_DECR_WRAP,
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GLVertexInfo {
    pub(crate) type_: GLenum,
    pub(crate) count: GLint,
    pub(crate) normalized: GLboolean,
}

pub(crate) fn oreVertexFormatToGL(format: VertexFormat) -> GLVertexInfo {
    match format {
        VertexFormat::float1 => GLVertexInfo {
            type_: GL_FLOAT,
            count: 1,
            normalized: GL_FALSE,
        },
        VertexFormat::float2 => GLVertexInfo {
            type_: GL_FLOAT,
            count: 2,
            normalized: GL_FALSE,
        },
        VertexFormat::float3 => GLVertexInfo {
            type_: GL_FLOAT,
            count: 3,
            normalized: GL_FALSE,
        },
        VertexFormat::float4 => GLVertexInfo {
            type_: GL_FLOAT,
            count: 4,
            normalized: GL_FALSE,
        },
        VertexFormat::uint8x4 => GLVertexInfo {
            type_: GL_UNSIGNED_BYTE,
            count: 4,
            normalized: GL_FALSE,
        },
        VertexFormat::sint8x4 => GLVertexInfo {
            type_: GL_BYTE,
            count: 4,
            normalized: GL_FALSE,
        },
        VertexFormat::unorm8x4 => GLVertexInfo {
            type_: GL_UNSIGNED_BYTE,
            count: 4,
            normalized: GL_TRUE,
        },
        VertexFormat::snorm8x4 => GLVertexInfo {
            type_: GL_BYTE,
            count: 4,
            normalized: GL_TRUE,
        },
        VertexFormat::uint16x2 => GLVertexInfo {
            type_: GL_UNSIGNED_SHORT,
            count: 2,
            normalized: GL_FALSE,
        },
        VertexFormat::sint16x2 => GLVertexInfo {
            type_: GL_SHORT,
            count: 2,
            normalized: GL_FALSE,
        },
        VertexFormat::unorm16x2 => GLVertexInfo {
            type_: GL_UNSIGNED_SHORT,
            count: 2,
            normalized: GL_TRUE,
        },
        VertexFormat::snorm16x2 => GLVertexInfo {
            type_: GL_SHORT,
            count: 2,
            normalized: GL_TRUE,
        },
        VertexFormat::uint16x4 => GLVertexInfo {
            type_: GL_UNSIGNED_SHORT,
            count: 4,
            normalized: GL_FALSE,
        },
        VertexFormat::sint16x4 => GLVertexInfo {
            type_: GL_SHORT,
            count: 4,
            normalized: GL_FALSE,
        },
        VertexFormat::float16x2 => GLVertexInfo {
            type_: GL_HALF_FLOAT,
            count: 2,
            normalized: GL_FALSE,
        },
        VertexFormat::float16x4 => GLVertexInfo {
            type_: GL_HALF_FLOAT,
            count: 4,
            normalized: GL_FALSE,
        },
        VertexFormat::uint32 => GLVertexInfo {
            type_: GL_UNSIGNED_INT,
            count: 1,
            normalized: GL_FALSE,
        },
    }
}

pub(crate) fn validate(pass: &RenderPassGLState) {
    // Source uses C assert(), which is compiled out by the pinned NDEBUG
    // WebGL2 release profile.
    debug_assert!(
        !nuxie_ore_metal::render_pass_is_finished(&pass.base),
        "RenderPass already finished"
    );
    debug_assert!(nuxie_ore_metal::render_pass_has_context(&pass.base));
}

fn requiredOwner<'a>(
    owner: Option<&'a AnyResourceHandle>,
    sourceContract: &'static str,
) -> &'a AnyResourceHandle {
    owner.unwrap_or_else(|| panic!("{sourceContract}"))
}

fn pipelineFromOwner<'a>(
    owner: &'a AnyResourceHandle,
    sourceContract: &'static str,
) -> &'a PipelineGL {
    owner
        .downcast_ref::<PipelineGL>()
        .unwrap_or_else(|| panic!("{sourceContract}"))
}

fn currentPipeline(pass: &RenderPassGLState) -> &PipelineGL {
    debug_assert!(pass.m_currentPipeline.is_some());
    let owner = pass
        .m_currentPipeline
        .as_ref()
        .expect("setPipeline must be called first");
    pipelineFromOwner(owner, "RenderPassGL current pipeline must be PipelineGL")
}

fn requireCompatibleExecution(
    pass: &RenderPassGLState,
    resource: &GLExecutionStamp,
    sourceContract: &'static str,
) {
    assert!(
        pass.executionStamp().sameDomain(resource),
        "{sourceContract}"
    );
}

pub(crate) fn setPipeline(pass: &mut RenderPassGLState, pipelineOwner: Option<&AnyResourceHandle>) {
    validate(pass);
    let pipelineOwner = requiredOwner(pipelineOwner, "RenderPassGL source pipeline");
    let pipeline = pipelineOwner
        .pipelineBase()
        .expect("RenderPassGL setPipeline requires a Pipeline resource");

    if !nuxie_ore_metal::render_pass_check_pipeline_compat(&pass.base, pipeline) {
        return;
    }

    let glPipeline = pipelineFromOwner(pipelineOwner, "RenderPassGL requires PipelineGL");
    requireCompatibleExecution(
        pass,
        glPipeline.executionStamp(),
        "RenderPassGL pipeline belongs to a foreign GL domain or generation",
    );
    *pass.m_currentPipeline = Some(pipelineOwner.clone());
    let desc = pipeline.desc();

    recordGLCommand(GLCommand::UseProgram(glPipeline.m_glProgram));

    if desc.cullMode == nuxie_ore_metal::types::CullMode::none {
        recordGLCommand(GLCommand::Disable(GL_CULL_FACE));
    } else {
        recordGLCommand(GLCommand::Enable(GL_CULL_FACE));
        recordGLCommand(GLCommand::CullFace(
            if desc.cullMode == nuxie_ore_metal::types::CullMode::front {
                GL_FRONT
            } else {
                GL_BACK
            },
        ));
    }

    // Inverted exactly to compensate for the authored WGSL-to-GLSL Y flip.
    recordGLCommand(GLCommand::FrontFace(
        if desc.winding == nuxie_ore_metal::types::FaceWinding::counterClockwise {
            GL_CW
        } else {
            GL_CCW
        },
    ));

    if desc.depthStencil.depthCompare != CompareFunction::always
        || desc.depthStencil.depthWriteEnabled
    {
        recordGLCommand(GLCommand::Enable(GL_DEPTH_TEST));
        recordGLCommand(GLCommand::DepthFunc(oreCompareFunctionToGL(
            desc.depthStencil.depthCompare,
        )));
        recordGLCommand(GLCommand::DepthMask(desc.depthStencil.depthWriteEnabled));
    } else {
        recordGLCommand(GLCommand::Disable(GL_DEPTH_TEST));
    }

    if desc.depthStencil.depthBias != 0 || desc.depthStencil.depthBiasSlopeScale != 0.0 {
        recordGLCommand(GLCommand::Enable(GL_POLYGON_OFFSET_FILL));
        recordGLCommand(GLCommand::PolygonOffset(
            desc.depthStencil.depthBiasSlopeScale,
            desc.depthStencil.depthBias as f32,
        ));
    } else {
        recordGLCommand(GLCommand::Disable(GL_POLYGON_OFFSET_FILL));
    }

    let hasStencil = desc.stencilFront.compare != CompareFunction::always
        || desc.stencilFront.failOp != StencilOp::keep
        || desc.stencilFront.depthFailOp != StencilOp::keep
        || desc.stencilFront.passOp != StencilOp::keep
        || desc.stencilBack.compare != CompareFunction::always
        || desc.stencilBack.failOp != StencilOp::keep
        || desc.stencilBack.depthFailOp != StencilOp::keep
        || desc.stencilBack.passOp != StencilOp::keep;

    if hasStencil {
        recordGLCommand(GLCommand::Enable(GL_STENCIL_TEST));
        recordGLCommand(GLCommand::StencilMaskSeparate(
            GL_FRONT,
            u32::from(desc.stencilWriteMask),
        ));
        recordGLCommand(GLCommand::StencilMaskSeparate(
            GL_BACK,
            u32::from(desc.stencilWriteMask),
        ));
        recordGLCommand(GLCommand::StencilFuncSeparate(
            GL_FRONT,
            oreCompareFunctionToGL(desc.stencilFront.compare),
            pass.m_glStencilRef as i32,
            u32::from(desc.stencilReadMask),
        ));
        recordGLCommand(GLCommand::StencilFuncSeparate(
            GL_BACK,
            oreCompareFunctionToGL(desc.stencilBack.compare),
            pass.m_glStencilRef as i32,
            u32::from(desc.stencilReadMask),
        ));
        recordGLCommand(GLCommand::StencilOpSeparate(
            GL_FRONT,
            oreStencilOpToGL(desc.stencilFront.failOp),
            oreStencilOpToGL(desc.stencilFront.depthFailOp),
            oreStencilOpToGL(desc.stencilFront.passOp),
        ));
        recordGLCommand(GLCommand::StencilOpSeparate(
            GL_BACK,
            oreStencilOpToGL(desc.stencilBack.failOp),
            oreStencilOpToGL(desc.stencilBack.depthFailOp),
            oreStencilOpToGL(desc.stencilBack.passOp),
        ));
    } else {
        recordGLCommand(GLCommand::Disable(GL_STENCIL_TEST));
    }

    if desc.colorCount > 0 && desc.colorTargets[0].blendEnabled {
        recordGLCommand(GLCommand::Enable(GL_BLEND));
        let blend = &desc.colorTargets[0].blend;
        recordGLCommand(GLCommand::BlendFuncSeparate(
            oreBlendFactorToGL(blend.srcColor),
            oreBlendFactorToGL(blend.dstColor),
            oreBlendFactorToGL(blend.srcAlpha),
            oreBlendFactorToGL(blend.dstAlpha),
        ));
        recordGLCommand(GLCommand::BlendEquationSeparate(
            oreBlendOpToGL(blend.colorOp),
            oreBlendOpToGL(blend.alphaOp),
        ));
    } else {
        recordGLCommand(GLCommand::Disable(GL_BLEND));
    }

    if desc.colorCount > 0 {
        let mask = desc.colorTargets[0].writeMask;
        recordGLCommand(GLCommand::ColorMask(
            (mask & ColorWriteMask::red) != ColorWriteMask::none,
            (mask & ColorWriteMask::green) != ColorWriteMask::none,
            (mask & ColorWriteMask::blue) != ColorWriteMask::none,
            (mask & ColorWriteMask::alpha) != ColorWriteMask::none,
        ));
    }
}

pub(crate) fn setVertexBuffer(
    pass: &mut RenderPassGLState,
    slot: u32,
    bufferOwner: Option<&AnyResourceHandle>,
    offset: u32,
) {
    validate(pass);
    debug_assert!(pass.m_currentPipeline.is_some());
    let pipelineOwner = pass
        .m_currentPipeline
        .as_ref()
        .expect("setPipeline must be called first");
    let pipeline = pipelineFromOwner(
        pipelineOwner,
        "RenderPassGL current pipeline must be PipelineGL",
    );
    debug_assert!((slot as usize) < pipeline.base.desc().vertexBuffers.len());
    let layout = &pipeline.base.desc().vertexBuffers[slot as usize];
    let bufferOwner = requiredOwner(bufferOwner, "RenderPassGL source vertex buffer");
    let glBuffer = bufferOwner
        .downcast_ref::<BufferGL>()
        .expect("RenderPassGL vertex buffer must be BufferGL");
    requireCompatibleExecution(
        pass,
        glBuffer.executionStamp(),
        "RenderPassGL vertex buffer belongs to a foreign GL domain or generation",
    );

    recordGLCommand(GLCommand::BindBuffer(GL_ARRAY_BUFFER, glBuffer.m_glBuffer));

    for attribute in &layout.attributes {
        let info = oreVertexFormatToGL(attribute.format);
        recordGLCommand(GLCommand::EnableVertexAttribArray(attribute.shaderSlot));

        let isIntType = matches!(
            info.type_,
            GL_UNSIGNED_INT | GL_INT | GL_UNSIGNED_BYTE | GL_BYTE | GL_UNSIGNED_SHORT | GL_SHORT
        ) && info.normalized == GL_FALSE;
        let attributeOffset = attribute.offset.wrapping_add(offset);

        if isIntType {
            recordGLCommand(GLCommand::VertexAttribIPointer {
                index: attribute.shaderSlot,
                size: info.count,
                type_: info.type_,
                stride: layout.stride as i32,
                offset: attributeOffset,
            });
        } else {
            recordGLCommand(GLCommand::VertexAttribPointer {
                index: attribute.shaderSlot,
                size: info.count,
                type_: info.type_,
                normalized: info.normalized,
                stride: layout.stride as i32,
                offset: attributeOffset,
            });
        }

        recordGLCommand(GLCommand::VertexAttribDivisor(
            attribute.shaderSlot,
            if layout.stepMode == VertexStepMode::instance {
                1
            } else {
                0
            },
        ));

        if !pass.m_usedAttribs || attribute.shaderSlot > pass.m_maxAttribSlot {
            pass.m_maxAttribSlot = attribute.shaderSlot;
        }
        pass.m_usedAttribs = true;
    }
}

pub(crate) fn setIndexBuffer(
    pass: &mut RenderPassGLState,
    bufferOwner: Option<&AnyResourceHandle>,
    format: IndexFormat,
    offset: u32,
) {
    validate(pass);
    let bufferOwner = requiredOwner(bufferOwner, "RenderPassGL source index buffer");
    let glBuffer = bufferOwner
        .downcast_ref::<BufferGL>()
        .expect("RenderPassGL index buffer must be BufferGL");
    requireCompatibleExecution(
        pass,
        glBuffer.executionStamp(),
        "RenderPassGL index buffer belongs to a foreign GL domain or generation",
    );
    recordGLCommand(GLCommand::BindBuffer(
        GL_ELEMENT_ARRAY_BUFFER,
        glBuffer.m_glBuffer,
    ));
    pass.m_glIndexFormat = format;
    // Authored GL behavior ignores the setIndexBuffer byte offset.
    let _ = offset;
}

pub(crate) fn setBindGroup(
    pass: &mut RenderPassGLState,
    groupIndex: u32,
    bindGroupOwner: Option<&AnyResourceHandle>,
    dynamicOffsets: Option<&[u32]>,
    dynamicOffsetCount: u32,
) {
    validate(pass);
    debug_assert!(bindGroupOwner.is_some());
    let bindGroupOwner = requiredOwner(bindGroupOwner, "RenderPassGL source bind group");

    let glBindGroup = bindGroupOwner
        .downcast_ref::<BindGroupGL>()
        .expect("RenderPassGL bind group must be BindGroupGL");
    requireCompatibleExecution(
        pass,
        glBindGroup.executionStamp(),
        "RenderPassGL bind group belongs to a foreign GL domain or generation",
    );

    // Source retains the group before consuming its bindings. The Rust
    // execution-domain check above is the mandatory safety boundary before
    // this ownership mutation.
    nuxie_ore_metal::render_pass_retain_bound_group(
        &mut pass.base,
        groupIndex,
        bindGroupOwner.clone(),
    );
    let mut dynamicIndex = 0_u32;

    for uniform in glBindGroup.m_glUBOs.iter() {
        let mut bindingOffset = uniform.offset;
        if uniform.hasDynamicOffset && dynamicIndex < dynamicOffsetCount {
            let offsets = dynamicOffsets.expect(
                "RenderPassGL dynamic-offset pointer must be non-null when its count is nonzero",
            );
            bindingOffset = bindingOffset.wrapping_add(offsets[dynamicIndex as usize]);
            dynamicIndex = dynamicIndex.wrapping_add(1);
        }
        recordGLCommand(GLCommand::BindBufferRange {
            target: GL_UNIFORM_BUFFER,
            index: uniform.slot,
            buffer: uniform.buffer,
            offset: bindingOffset,
            size: uniform.size,
        });
    }

    for texture in glBindGroup.m_glTextures.iter() {
        recordGLCommand(GLCommand::ActiveTexture(
            GL_TEXTURE0.wrapping_add(texture.slot),
        ));
        recordGLCommand(GLCommand::BindTexture(texture.target, texture.texture));

        if !pass.m_usedSamplers || texture.slot > pass.m_maxSamplerSlot {
            pass.m_maxSamplerSlot = texture.slot;
        }
        pass.m_usedSamplers = true;
    }

    for sampler in glBindGroup.m_glSamplers.iter() {
        recordGLCommand(GLCommand::BindSampler(sampler.slot, sampler.sampler));

        if !pass.m_usedSamplers || sampler.slot > pass.m_maxSamplerSlot {
            pass.m_maxSamplerSlot = sampler.slot;
        }
        pass.m_usedSamplers = true;
    }
}

pub(crate) fn setViewport(
    pass: &mut RenderPassGLState,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    minDepth: f32,
    maxDepth: f32,
) {
    validate(pass);
    pass.m_viewportWidth = width as u32;
    pass.m_viewportHeight = height as u32;
    recordGLCommand(GLCommand::Viewport(
        x as i32,
        y as i32,
        width as i32,
        height as i32,
    ));
    recordGLCommand(GLCommand::DepthRange(minDepth, maxDepth));
}

pub(crate) fn setScissorRect(
    pass: &mut RenderPassGLState,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) {
    validate(pass);
    recordGLCommand(GLCommand::Enable(GL_SCISSOR_TEST));
    recordGLCommand(GLCommand::Scissor(x, y, width, height));
}

pub(crate) fn setStencilReference(pass: &mut RenderPassGLState, reference: u32) {
    validate(pass);
    pass.m_glStencilRef = reference;
    if let Some(pipelineOwner) = pass.m_currentPipeline.as_ref() {
        let pipeline = pipelineFromOwner(
            pipelineOwner,
            "RenderPassGL current pipeline must be PipelineGL",
        );
        let desc = pipeline.base.desc();
        recordGLCommand(GLCommand::StencilFuncSeparate(
            GL_FRONT,
            oreCompareFunctionToGL(desc.stencilFront.compare),
            reference as i32,
            u32::from(desc.stencilReadMask),
        ));
        recordGLCommand(GLCommand::StencilFuncSeparate(
            GL_BACK,
            oreCompareFunctionToGL(desc.stencilBack.compare),
            reference as i32,
            u32::from(desc.stencilReadMask),
        ));
    }
}

pub(crate) fn setBlendColor(
    pass: &mut RenderPassGLState,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
) {
    validate(pass);
    recordGLCommand(GLCommand::BlendColor(red, green, blue, alpha));
}

pub(crate) fn draw(
    pass: &mut RenderPassGLState,
    vertexCount: u32,
    instanceCount: u32,
    firstVertex: u32,
    firstInstance: u32,
) {
    validate(pass);
    let pipeline = currentPipeline(pass);
    let mode = oreTopologyToGL(pipeline.base.desc().topology);

    // drawBaseInstance is false for the pinned GL backend. The source Lua
    // guard rejects nonzero firstInstance, and this method discards it.
    let _ = firstInstance;

    if instanceCount > 1 {
        recordGLCommand(GLCommand::DrawArraysInstanced {
            mode,
            first: firstVertex,
            count: vertexCount,
            instanceCount,
        });
    } else {
        recordGLCommand(GLCommand::DrawArrays {
            mode,
            first: firstVertex,
            count: vertexCount,
        });
    }
}

pub(crate) fn drawIndexed(
    pass: &mut RenderPassGLState,
    indexCount: u32,
    instanceCount: u32,
    firstIndex: u32,
    baseVertex: i32,
    firstInstance: u32,
) {
    validate(pass);
    let pipeline = currentPipeline(pass);
    let mode = oreTopologyToGL(pipeline.base.desc().topology);
    let indexType = if pass.m_glIndexFormat == IndexFormat::uint32 {
        GL_UNSIGNED_INT
    } else {
        GL_UNSIGNED_SHORT
    };
    let indexSize = if indexType == GL_UNSIGNED_INT { 4 } else { 2 };
    let offset = firstIndex.wrapping_mul(indexSize);

    // The source GL capability contract rejects these before dispatch and the
    // implementation deliberately discards both values.
    let _ = baseVertex;
    let _ = firstInstance;

    if instanceCount > 1 {
        recordGLCommand(GLCommand::DrawElementsInstanced {
            mode,
            count: indexCount,
            type_: indexType,
            offset,
            instanceCount,
        });
    } else {
        recordGLCommand(GLCommand::DrawElements {
            mode,
            count: indexCount,
            type_: indexType,
            offset,
        });
    }
}

pub(crate) fn finish(pass: &mut RenderPassGLState) {
    if nuxie_ore_metal::render_pass_is_finished(&pass.base) {
        return;
    }
    nuxie_ore_metal::render_pass_set_finished(&mut pass.base, true);

    // Preserve the authored release sequence: current pipeline, then every
    // retained base bind group, before any GL unbinding or native deletion.
    *pass.m_currentPipeline = None;
    nuxie_ore_metal::render_pass_clear_bound_groups(&mut pass.base);

    recordGLCommand(GLCommand::Disable(GL_SCISSOR_TEST));
    recordGLCommand(GLCommand::Disable(GL_BLEND));
    recordGLCommand(GLCommand::Disable(GL_DEPTH_TEST));
    recordGLCommand(GLCommand::Disable(GL_STENCIL_TEST));
    recordGLCommand(GLCommand::Disable(GL_CULL_FACE));
    recordGLCommand(GLCommand::Disable(GL_POLYGON_OFFSET_FILL));
    recordGLCommand(GLCommand::DepthMask(true));
    recordGLCommand(GLCommand::ColorMask(true, true, true, true));

    if pass.m_usedSamplers {
        for slot in 0..=pass.m_maxSamplerSlot {
            recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0.wrapping_add(slot)));
            recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_2D, 0));
            recordGLCommand(GLCommand::BindTexture(GL_TEXTURE_CUBE_MAP, 0));
            recordGLCommand(GLCommand::BindSampler(slot, 0));
        }
        recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0));
    }

    if pass.m_usedAttribs {
        for slot in 0..=pass.m_maxAttribSlot {
            recordGLCommand(GLCommand::DisableVertexAttribArray(slot));
        }
    }

    recordGLCommand(GLCommand::BindBuffer(GL_ARRAY_BUFFER, 0));

    if pass.m_ownsVAO && pass.m_glVAO != 0 {
        recordGLCommand(GLCommand::DeleteVertexArray(pass.m_glVAO));
        pass.m_glVAO = 0;
    }
    recordGLCommand(GLCommand::BindVertexArray(pass.m_prevVAO));

    if pass.m_glResolveCount > 0 {
        let resolveFBO = generateGLObject(GLObjectKind::Framebuffer);
        for resolveIndex in 0..pass.m_glResolveCount {
            let resolve = pass.m_glResolves[resolveIndex as usize];
            recordGLCommand(GLCommand::BindFramebuffer(
                GL_READ_FRAMEBUFFER,
                pass.m_glFBO,
            ));
            recordGLCommand(GLCommand::ReadBuffer(
                GL_COLOR_ATTACHMENT0.wrapping_add(resolve.colorIndex),
            ));
            recordGLCommand(GLCommand::BindFramebuffer(GL_DRAW_FRAMEBUFFER, resolveFBO));
            recordGLCommand(GLCommand::FramebufferTexture2D {
                target: GL_DRAW_FRAMEBUFFER,
                attachment: GL_COLOR_ATTACHMENT0,
                texture_target: resolve.resolveTarget,
                texture: resolve.resolveTex,
                level: 0,
            });
            recordGLCommand(GLCommand::BlitFramebuffer(
                [
                    0,
                    0,
                    resolve.width as i32,
                    resolve.height as i32,
                    0,
                    0,
                    resolve.width as i32,
                    resolve.height as i32,
                ],
                GL_COLOR_BUFFER_BIT,
                GL_NEAREST,
            ));
        }
        recordGLCommand(GLCommand::DeleteFramebuffer(resolveFBO));
    }

    if pass.m_ownsFBO && pass.m_glFBO != 0 {
        recordGLCommand(GLCommand::DeleteFramebuffer(pass.m_glFBO));
    }
    recordGLCommand(GLCommand::BindFramebuffer(GL_FRAMEBUFFER, pass.m_prevFBO));

    nuxie_ore_metal::render_pass_clear_context(&mut pass.base);
}

/// Context-loss teardown for a pass whose creation generation is no longer
/// executable. Numeric GL names are intentionally abandoned; the retained
/// Rust resource graph and inherited context reference still unwind in source
/// ownership order.
pub(crate) fn abandonAfterContextLoss(pass: &mut RenderPassGLState) {
    if nuxie_ore_metal::render_pass_is_finished(&pass.base) {
        return;
    }
    nuxie_ore_metal::render_pass_set_finished(&mut pass.base, true);
    *pass.m_currentPipeline = None;
    nuxie_ore_metal::render_pass_clear_bound_groups(&mut pass.base);
    pass.m_glFBO = 0;
    pass.m_glVAO = 0;
    pass.m_ownsFBO = false;
    pass.m_ownsVAO = false;
    pass.m_glResolveCount = 0;
    nuxie_ore_metal::render_pass_clear_context(&mut pass.base);
}

pub(crate) const SOURCE_STATIC_MAPPING_COUNT: usize = 6;
pub(crate) const SOURCE_MAPPING_CASE_COUNT: usize = 57;
pub(crate) const SOURCE_METHOD_DEFINITION_COUNT: usize = 13;
pub(crate) const SOURCE_GL_CALL_SITE_COUNT: usize = 73;
pub(crate) const SOURCE_ASSERT_COUNT: usize = 11;
pub(crate) const SOURCE_IF_COUNT: usize = 23;
pub(crate) const SOURCE_LOOP_COUNT: usize = 8;
const _: [(); 20041] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_ore_metal::gpu_resource::{GPUResource, GpuResourcePayload, ResourceHandle};
    use nuxie_ore_metal::pipeline::Pipeline;
    use nuxie_ore_metal::types::{Features, PipelineDesc};
    use std::cell::RefCell;
    use std::mem::ManuallyDrop;
    use std::rc::Rc;

    struct CommandProvider {
        commands: Rc<RefCell<Vec<GLCommand>>>,
    }

    impl GLExecutionProvider for CommandProvider {
        fn installContextLifecycleIngress(&mut self, _ingress: GLContextLifecycleIngress) {}

        fn installFinalReleaseIngress(
            &mut self,
            _ingress: GLFinalReleaseIngress,
        ) -> std::sync::Arc<dyn nuxie_ore_metal::gpu_resource::ResourceFinalReleaseWake> {
            std::sync::Arc::new(TestFinalReleaseWake::default())
        }

        fn submit(&mut self, command: GLCommand) {
            self.commands.borrow_mut().push(command);
        }

        fn generateObject(&mut self, _kind: GLObjectKind) -> GLuint {
            1
        }

        fn createProgram(&mut self) -> GLuint {
            1
        }

        fn createShader(&mut self, _shaderType: GLenum) -> GLuint {
            1
        }

        fn getInteger(&mut self, _parameter: GLenum) -> GLint {
            0
        }

        fn getString(&mut self, _parameter: GLenum) -> Option<Vec<u8>> {
            None
        }

        fn getExtension(&mut self, _index: GLuint) -> Option<Vec<u8>> {
            None
        }

        fn enableWebGLExtension(&mut self, _name: &str) -> bool {
            false
        }

        fn isObject(&mut self, _kind: GLObjectKind, _name: GLuint) -> bool {
            false
        }

        fn checkFramebufferStatus(&mut self, _target: GLenum) -> GLenum {
            GL_FRAMEBUFFER_COMPLETE
        }

        fn shaderParameter(&mut self, _shader: GLuint, _parameter: GLenum) -> GLint {
            0
        }

        fn shaderInfoLog(&mut self, _shader: GLuint, _maxLength: usize) -> Vec<u8> {
            Vec::new()
        }

        fn programParameter(&mut self, _program: GLuint, _parameter: GLenum) -> GLint {
            0
        }

        fn programInfoLog(&mut self, _program: GLuint, _maxLength: usize) -> Vec<u8> {
            Vec::new()
        }

        fn uniformBlockIndex(&mut self, _program: GLuint, _name: &[u8]) -> GLuint {
            0
        }

        fn uniformLocation(&mut self, _program: GLuint, _name: &[u8]) -> GLint {
            -1
        }

        fn readPixelsRGBA8(&mut self, _x: i32, _y: i32, _width: u32, _height: u32) -> Vec<u8> {
            Vec::new()
        }

        fn contextLost(&mut self, _nextGeneration: u64) {}
    }

    /// Test-only foreign backend pipeline with the exact source offset-zero
    /// Pipeline base. It exercises the base-kind projection without making
    /// RenderPassGL depend on another concrete backend module.
    #[repr(C)]
    struct ForeignPipeline {
        base: ManuallyDrop<Pipeline>,
    }

    impl Drop for ForeignPipeline {
        fn drop(&mut self) {
            unsafe { ManuallyDrop::drop(&mut self.base) };
        }
    }

    unsafe impl Send for ForeignPipeline {}

    unsafe impl GpuResourcePayload for ForeignPipeline {
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

    #[test]
    fn incompatible_foreign_pipeline_returns_before_gl_downcast_and_retain() {
        let context = nuxie_ore_metal::new_context_backend_base(Features::default(), None);
        let mut pass = RenderPassGLState::newUnstamped(&context);
        let foreign = ForeignPipeline {
            // Default Pipeline has one color target; the default pass base has
            // zero, so the source compatibility check must return false.
            base:
                ManuallyDrop::new(
                    nuxie_ore_metal::new_pipeline_backend_base_without_manager(
                        &PipelineDesc::default(),
                    )
                    .expect("default PipelineDesc is valid"),
                ),
        };
        let foreign = ResourceHandle::new(None, foreign).erase();

        resetGLCommandStream();
        setPipeline(&mut pass, Some(&foreign));

        assert!(pass.m_currentPipeline.is_none());
        assert_eq!(foreign.debugging_refcnt(), 1);
        assert!(takeGLCommands().is_empty());
        assert_eq!(
            context.lastError(),
            "setPipeline: pipeline has 1 color targets but render pass was begun with 0"
        );

        finish(&mut pass);
        let _ = takeGLCommands();
    }

    #[test]
    fn finish_restores_actual_previous_vao_and_fbo_names() {
        let mut pass = RenderPassGLState::withoutContext();
        pass.m_prevVAO = 73;
        pass.m_prevFBO = 89;

        resetGLCommandStream();
        finish(&mut pass);
        let commands = takeGLCommands();

        assert!(commands.contains(&GLCommand::BindVertexArray(73)));
        assert!(commands.contains(&GLCommand::BindFramebuffer(GL_FRAMEBUFFER, 89)));
        assert!(!commands.iter().any(|command| matches!(
            command,
            GLCommand::BindVertexArrayFromQuery(_) | GLCommand::BindFramebufferFromQuery(_, _)
        )));
    }

    #[test]
    fn finish_deletes_sole_owned_pipeline_before_disabling_and_unbinding_state() {
        const PROGRAM: GLuint = 91;

        let commands = Rc::new(RefCell::new(Vec::new()));
        let domain = GLExecutionDomain::new(Box::new(CommandProvider {
            commands: Rc::clone(&commands),
        }));
        let context = nuxie_ore_metal::new_context_backend_base_with_final_release_drain(
            Features::default(),
            None,
            domain.resourceFinalReleaseDrain(),
        );
        let mut pass = RenderPassGLState::new(&context, domain.stamp());
        let mut pipeline = PipelineGL::new(&PipelineDesc::default(), domain.stamp())
            .expect("default pipeline descriptor is valid");
        pipeline.m_glProgram = PROGRAM;
        let pipeline = ResourceHandle::new_in_domain(
            None,
            nuxie_ore_metal::context_backend_domain(&context),
            pipeline,
        )
        .erase();

        // The pass is the pipeline's sole owner. Source `finish()` clears this
        // reference before issuing any state teardown, so `~PipelineGL()` must
        // synchronously delete its program before the first disable/unbind.
        *pass.m_currentPipeline = Some(pipeline);
        domain.withCurrent(|| finish(&mut pass));

        let commands = commands.borrow();
        assert_eq!(commands.first(), Some(&GLCommand::DeleteProgram(PROGRAM)));
        assert_eq!(commands.get(1), Some(&GLCommand::Disable(GL_SCISSOR_TEST)));

        let deleteProgram = commands
            .iter()
            .position(|command| *command == GLCommand::DeleteProgram(PROGRAM))
            .expect("sole-owned pipeline program is deleted during finish");
        for teardown in [
            GLCommand::Disable(GL_BLEND),
            GLCommand::Disable(GL_DEPTH_TEST),
            GLCommand::Disable(GL_STENCIL_TEST),
            GLCommand::Disable(GL_CULL_FACE),
            GLCommand::Disable(GL_POLYGON_OFFSET_FILL),
            GLCommand::BindBuffer(GL_ARRAY_BUFFER, 0),
            GLCommand::BindVertexArray(0),
            GLCommand::BindFramebuffer(GL_FRAMEBUFFER, 0),
        ] {
            let teardownIndex = commands
                .iter()
                .position(|command| *command == teardown)
                .expect("finish emits every authored state teardown command");
            assert!(
                deleteProgram < teardownIndex,
                "pipeline deletion must precede {teardown:?}"
            );
        }
    }
}
