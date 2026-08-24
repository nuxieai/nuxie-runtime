//! Complete mechanical implementation translation of
//! `renderer/src/gl/gl_state.cpp` for the frozen WebGL2 configuration.

#![allow(non_snake_case)]

use super::gl_state_decl::{GLState, ScissorAction, ValidState};
use super::gles3_decl::*;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    BlendEquation, CullFace, IAABB, PipelineState, StencilCompareOp, StencilOp, AABBu16,
};

pub(crate) const PINNED_SOURCE: &str = include_str!("source/renderer_src_gl_gl_state.cpp");

pub(crate) fn invalidate(state: &mut GLState) {
    state.m_validState = ValidState::default();
    recordGLCommand(GLCommand::FrontFace(GL_CW));
    recordGLCommand(GLCommand::DepthRange(0.0, 1.0));
    recordGLCommand(GLCommand::DepthFunc(GL_LESS));
    recordGLCommand(GLCommand::ClearDepth(1.0));
    recordGLCommand(GLCommand::ClearStencil(0));
    recordGLCommand(GLCommand::Disable(GL_DITHER));
    recordGLCommand(GLCommand::Disable(GL_POLYGON_OFFSET_FILL));
    recordGLCommand(GLCommand::Disable(GL_RASTERIZER_DISCARD));
    recordGLCommand(GLCommand::Disable(GL_SAMPLE_ALPHA_TO_COVERAGE));
    recordGLCommand(GLCommand::Disable(GL_SAMPLE_COVERAGE));
    for (parameter, value) in [
        (GL_UNPACK_ROW_LENGTH, 0),
        (GL_UNPACK_SKIP_ROWS, 0),
        (GL_UNPACK_SKIP_PIXELS, 0),
        (GL_UNPACK_ALIGNMENT, 4),
        (GL_PACK_ROW_LENGTH, 0),
        (GL_PACK_SKIP_ROWS, 0),
        (GL_PACK_SKIP_PIXELS, 0),
        (GL_PACK_ALIGNMENT, 4),
    ] {
        recordGLCommand(GLCommand::PixelStore(parameter, value));
    }
    recordGLCommand(GLCommand::BindBuffer(GL_PIXEL_UNPACK_BUFFER, 0));
    if state.m_capabilities.ANGLE_provoking_vertex() {
        recordGLCommand(GLCommand::ProvokingVertex(
            GL_FIRST_VERTEX_CONVENTION_ANGLE,
        ));
    }
}

pub(crate) fn setScissor(state: &mut GLState, scissor: IAABB, renderTargetHeight: u32) {
    assert!(scissor.left >= 0);
    assert!(scissor.right >= scissor.left);
    assert!(scissor.top >= 0);
    assert!(scissor.bottom >= scissor.top);
    let bottom = u32::try_from(scissor.bottom).expect("nonnegative scissor bottom");
    setScissorRaw(
        state,
        scissor.left as u32,
        renderTargetHeight
            .checked_sub(bottom)
            .expect("scissor is inside render target"),
        (scissor.right - scissor.left) as u32,
        (scissor.bottom - scissor.top) as u32,
    );
}

pub(crate) fn setScissorU16(
    state: &mut GLState,
    scissor: AABBu16,
    renderTargetHeight: u32,
) {
    assert!(scissor.right >= scissor.left);
    assert!(scissor.bottom >= scissor.top);
    setScissorRaw(
        state,
        scissor.left as u32,
        renderTargetHeight
            .checked_sub(scissor.bottom as u32)
            .expect("scissor is inside render target"),
        (scissor.right - scissor.left) as u32,
        (scissor.bottom - scissor.top) as u32,
    );
}

pub(crate) fn setScissorRaw(
    state: &mut GLState,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
) {
    let box_ = [left, top, width, height];
    if !state.m_validState.scissorBox() || state.m_scissorBox != box_ {
        recordGLCommand(GLCommand::Scissor(left, top, width, height));
        state.m_scissorBox = box_;
        state.m_validState.setScissorBox(true);
    }
    if !state.m_validState.scissorEnabled() || !state.m_scissorEnabled {
        recordGLCommand(GLCommand::Enable(GL_SCISSOR_TEST));
        state.m_scissorEnabled = true;
        state.m_validState.setScissorEnabled(true);
    }
}

pub(crate) fn disableScissor(state: &mut GLState) {
    if !state.m_validState.scissorEnabled() || state.m_scissorEnabled {
        recordGLCommand(GLCommand::Disable(GL_SCISSOR_TEST));
        state.m_scissorEnabled = false;
        state.m_validState.setScissorEnabled(true);
    }
}

fn glEnableDisable(value: GLenum, enabled: bool) {
    recordGLCommand(if enabled {
        GLCommand::Enable(value)
    } else {
        GLCommand::Disable(value)
    });
}

pub(crate) fn setDepthStencilEnabled(
    state: &mut GLState,
    depthEnabled: bool,
    stencilEnabled: bool,
) {
    if !state.m_validState.depthStencilEnabled() || state.m_depthTestEnabled != depthEnabled {
        glEnableDisable(GL_DEPTH_TEST, depthEnabled);
        state.m_depthTestEnabled = depthEnabled;
    }
    if !state.m_validState.depthStencilEnabled() || state.m_stencilTestEnabled != stencilEnabled {
        glEnableDisable(GL_STENCIL_TEST, stencilEnabled);
        state.m_stencilTestEnabled = stencilEnabled;
    }
    state.m_validState.setDepthStencilEnabled(true);
}

pub(crate) fn setCullFace(state: &mut GLState, cullFace: GLenum) {
    if !state.m_validState.cullFace() || cullFace != state.m_cullFace {
        if cullFace == GL_NONE {
            recordGLCommand(GLCommand::Disable(GL_CULL_FACE));
        } else {
            if !state.m_validState.cullFace() || state.m_cullFace == GL_NONE {
                recordGLCommand(GLCommand::Enable(GL_CULL_FACE));
            }
            recordGLCommand(GLCommand::CullFace(cullFace));
        }
        state.m_cullFace = cullFace;
        state.m_validState.setCullFace(true);
    }
}

fn glStencilOp(op: StencilOp) -> GLenum {
    match op {
        StencilOp::keep => GL_KEEP,
        StencilOp::replace => GL_REPLACE,
        StencilOp::zero => GL_ZERO,
        StencilOp::decrClamp => GL_DECR,
        StencilOp::incrWrap => GL_INCR_WRAP,
        StencilOp::decrWrap => GL_DECR_WRAP,
    }
}

fn glStencilFunc(compareOp: StencilCompareOp) -> GLenum {
    match compareOp {
        StencilCompareOp::less => GL_LESS,
        StencilCompareOp::equal => GL_EQUAL,
        StencilCompareOp::lessOrEqual => GL_LEQUAL,
        StencilCompareOp::notEqual => GL_NOTEQUAL,
        StencilCompareOp::always => GL_ALWAYS,
    }
}

fn glCullFace(riveCullFace: CullFace) -> GLenum {
    match riveCullFace {
        CullFace::none => GL_NONE,
        CullFace::clockwise => GL_FRONT,
        CullFace::counterclockwise => GL_BACK,
    }
}

pub(crate) fn setBlendEquation(state: &mut GLState, blendEquation: BlendEquation) {
    if state.m_validState.blendEquation() && blendEquation == state.m_blendEquation {
        return;
    }
    if !state.m_validState.blendEquation() || state.m_blendEquation == BlendEquation::none {
        recordGLCommand(GLCommand::Enable(GL_BLEND));
    }
    match blendEquation {
        BlendEquation::none => recordGLCommand(GLCommand::Disable(GL_BLEND)),
        BlendEquation::srcOver => {
            recordGLCommand(GLCommand::BlendEquation(GL_FUNC_ADD));
            recordGLCommand(GLCommand::BlendFunc(GL_ONE, GL_ONE_MINUS_SRC_ALPHA));
        }
        BlendEquation::plus => {
            recordGLCommand(GLCommand::BlendEquation(GL_FUNC_ADD));
            recordGLCommand(GLCommand::BlendFunc(GL_ONE, GL_ONE));
        }
        BlendEquation::min => {
            recordGLCommand(GLCommand::BlendEquation(GL_MIN));
            recordGLCommand(GLCommand::BlendFunc(GL_ONE, GL_ONE));
        }
        BlendEquation::max => {
            recordGLCommand(GLCommand::BlendEquation(GL_MAX));
            recordGLCommand(GLCommand::BlendFunc(GL_ONE, GL_ONE));
        }
        BlendEquation::screen => recordGLCommand(GLCommand::BlendEquation(GL_SCREEN_KHR)),
        BlendEquation::overlay => recordGLCommand(GLCommand::BlendEquation(GL_OVERLAY_KHR)),
        BlendEquation::darken => recordGLCommand(GLCommand::BlendEquation(GL_DARKEN_KHR)),
        BlendEquation::lighten => recordGLCommand(GLCommand::BlendEquation(GL_LIGHTEN_KHR)),
        BlendEquation::colorDodge => recordGLCommand(GLCommand::BlendEquation(GL_COLORDODGE_KHR)),
        BlendEquation::colorBurn => recordGLCommand(GLCommand::BlendEquation(GL_COLORBURN_KHR)),
        BlendEquation::hardLight => recordGLCommand(GLCommand::BlendEquation(GL_HARDLIGHT_KHR)),
        BlendEquation::softLight => recordGLCommand(GLCommand::BlendEquation(GL_SOFTLIGHT_KHR)),
        BlendEquation::difference => recordGLCommand(GLCommand::BlendEquation(GL_DIFFERENCE_KHR)),
        BlendEquation::exclusion => recordGLCommand(GLCommand::BlendEquation(GL_EXCLUSION_KHR)),
        BlendEquation::multiply => recordGLCommand(GLCommand::BlendEquation(GL_MULTIPLY_KHR)),
        BlendEquation::hue => recordGLCommand(GLCommand::BlendEquation(GL_HSL_HUE_KHR)),
        BlendEquation::saturation => {
            recordGLCommand(GLCommand::BlendEquation(GL_HSL_SATURATION_KHR))
        }
        BlendEquation::color => recordGLCommand(GLCommand::BlendEquation(GL_HSL_COLOR_KHR)),
        BlendEquation::luminosity => {
            recordGLCommand(GLCommand::BlendEquation(GL_HSL_LUMINOSITY_KHR))
        }
    }
    state.m_blendEquation = blendEquation;
    state.m_validState.setBlendEquation(true);
}

pub(crate) fn setWriteMasks(
    state: &mut GLState,
    colorWriteMask: bool,
    depthWriteMask: bool,
    stencilWriteMask: u8,
) {
    if !state.m_validState.writeMasks() {
        recordGLCommand(GLCommand::ColorMask(
            colorWriteMask,
            colorWriteMask,
            colorWriteMask,
            colorWriteMask,
        ));
        recordGLCommand(GLCommand::DepthMask(depthWriteMask));
        recordGLCommand(GLCommand::StencilMask(stencilWriteMask as u32));
        state.m_colorWriteMask = colorWriteMask;
        state.m_depthWriteMask = depthWriteMask;
        state.m_stencilWriteMask = stencilWriteMask as u32;
        state.m_validState.setWriteMasks(true);
    } else {
        if colorWriteMask != state.m_colorWriteMask {
            recordGLCommand(GLCommand::ColorMask(
                colorWriteMask,
                colorWriteMask,
                colorWriteMask,
                colorWriteMask,
            ));
            state.m_colorWriteMask = colorWriteMask;
        }
        if depthWriteMask != state.m_depthWriteMask {
            recordGLCommand(GLCommand::DepthMask(depthWriteMask));
            state.m_depthWriteMask = depthWriteMask;
        }
        if stencilWriteMask as u32 != state.m_stencilWriteMask {
            recordGLCommand(GLCommand::StencilMask(stencilWriteMask as u32));
            state.m_stencilWriteMask = stencilWriteMask as u32;
        }
    }
}

pub(crate) fn setPipelineState(
    state: &mut GLState,
    pipelineState: &PipelineState,
    scissorAction: ScissorAction,
) {
    match scissorAction {
        ScissorAction::disable => disableScissor(state),
        ScissorAction::ignore => {}
    }
    setDepthStencilEnabled(
        state,
        pipelineState.depthTestEnabled,
        pipelineState.stencilTestEnabled,
    );
    if pipelineState.stencilTestEnabled {
        let front = pipelineState.stencilFrontOps;
        if !pipelineState.stencilDoubleSided {
            recordGLCommand(GLCommand::StencilFunc(
                glStencilFunc(front.compareOp),
                pipelineState.stencilReference as i32,
                pipelineState.stencilCompareMask as u32,
            ));
            recordGLCommand(GLCommand::StencilOp(
                glStencilOp(front.stencilFailOp),
                glStencilOp(front.depthFailOp),
                glStencilOp(front.depthStencilPassOp),
            ));
        } else {
            for (face, ops) in [
                (GL_FRONT, pipelineState.stencilFrontOps),
                (GL_BACK, pipelineState.stencilBackOps),
            ] {
                recordGLCommand(GLCommand::StencilFuncSeparate(
                    face,
                    glStencilFunc(ops.compareOp),
                    pipelineState.stencilReference as i32,
                    pipelineState.stencilCompareMask as u32,
                ));
                recordGLCommand(GLCommand::StencilOpSeparate(
                    face,
                    glStencilOp(ops.stencilFailOp),
                    glStencilOp(ops.depthFailOp),
                    glStencilOp(ops.depthStencilPassOp),
                ));
            }
        }
    }
    setCullFace(state, glCullFace(pipelineState.cullFace));
    setBlendEquation(state, pipelineState.blendEquation);
    setWriteMasks(
        state,
        pipelineState.colorWriteEnabled,
        pipelineState.depthWriteEnabled,
        pipelineState.stencilWriteMask,
    );
}

pub(crate) fn bindProgram(state: &mut GLState, programID: GLuint) {
    if !state.m_validState.boundProgramID() || programID != state.m_boundProgramID {
        recordGLCommand(GLCommand::UseProgram(programID));
        state.m_boundProgramID = programID;
        state.m_validState.setBoundProgramID(true);
    }
}

pub(crate) fn bindVAO(state: &mut GLState, vao: GLuint) {
    if !state.m_validState.boundVAO() || vao != state.m_boundVAO {
        recordGLCommand(GLCommand::BindVertexArray(vao));
        state.m_boundVAO = vao;
        state.m_validState.setBoundVAO(true);
    }
}

pub(crate) fn bindBuffer(state: &mut GLState, target: GLenum, bufferID: GLuint) {
    match target {
        GL_ARRAY_BUFFER => {
            if !state.m_validState.boundArrayBufferID() || bufferID != state.m_boundArrayBufferID {
                recordGLCommand(GLCommand::BindBuffer(GL_ARRAY_BUFFER, bufferID));
                state.m_boundArrayBufferID = bufferID;
                state.m_validState.setBoundArrayBufferID(true);
            }
        }
        GL_UNIFORM_BUFFER => {
            if !state.m_validState.boundUniformBufferID() || bufferID != state.m_boundUniformBufferID {
                recordGLCommand(GLCommand::BindBuffer(GL_UNIFORM_BUFFER, bufferID));
                state.m_boundUniformBufferID = bufferID;
                state.m_validState.setBoundUniformBufferID(true);
            }
        }
        _ => recordGLCommand(GLCommand::BindBuffer(target, bufferID)),
    }
}

pub(crate) fn deleteProgram(state: &mut GLState, programID: GLuint) {
    recordGLCommand(GLCommand::DeleteProgram(programID));
    if state.m_validState.boundProgramID() && state.m_boundProgramID == programID {
        state.m_boundProgramID = 0;
    }
}

pub(crate) fn deleteVAO(state: &mut GLState, vao: GLuint) {
    recordGLCommand(GLCommand::DeleteVertexArray(vao));
    if state.m_validState.boundVAO() && state.m_boundVAO == vao {
        state.m_boundVAO = 0;
    }
}

pub(crate) fn deleteBuffer(state: &mut GLState, bufferID: GLuint) {
    recordGLCommand(GLCommand::DeleteBuffer(bufferID));
    if state.m_validState.boundArrayBufferID() && state.m_boundArrayBufferID == bufferID {
        state.m_boundArrayBufferID = 0;
    }
    if state.m_validState.boundUniformBufferID() && state.m_boundUniformBufferID == bufferID {
        state.m_boundUniformBufferID = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redundant_scissor_program_and_buffer_state_is_suppressed_exactly() {
        resetGLCommandStream();
        let mut state = GLState::new(GLCapabilities::default());
        takeGLCommands();
        state.setScissorRaw(1, 2, 3, 4);
        state.setScissorRaw(1, 2, 3, 4);
        state.bindProgram(7);
        state.bindProgram(7);
        state.bindBuffer(GL_ARRAY_BUFFER, 9);
        state.bindBuffer(GL_ARRAY_BUFFER, 9);
        assert_eq!(
            takeGLCommands(),
            vec![
                GLCommand::Scissor(1, 2, 3, 4),
                GLCommand::Enable(GL_SCISSOR_TEST),
                GLCommand::UseProgram(7),
                GLCommand::BindBuffer(GL_ARRAY_BUFFER, 9),
            ]
        );
    }

    #[test]
    fn complete_source_line_denominators_are_frozen() {
        assert_eq!(super::super::gl_state_decl::PINNED_SOURCE.lines().count(), 95);
        assert_eq!(PINNED_SOURCE.lines().count(), 491);
    }
}
