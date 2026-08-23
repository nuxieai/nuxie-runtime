//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/gl/gl_state.hpp`.

#![allow(non_camel_case_types, non_snake_case)]

use super::gles3_decl::{GLCapabilities, GLenum, GLuint};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    BlendEquation, IAABB, PipelineState, AABBu16,
};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_include_rive_renderer_gl_gl_state.hpp");

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScissorAction {
    disable = 0,
    ignore = 1,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ValidState {
    pub(crate) scissorBox: bool,
    pub(crate) scissorEnabled: bool,
    pub(crate) depthStencilEnabled: bool,
    pub(crate) writeMasks: bool,
    pub(crate) blendEquation: bool,
    pub(crate) cullFace: bool,
    pub(crate) boundProgramID: bool,
    pub(crate) boundVAO: bool,
    pub(crate) boundArrayBufferID: bool,
    pub(crate) boundUniformBufferID: bool,
    pub(crate) boundPixelUnpackBufferID: bool,
}

#[repr(C)]
pub(crate) struct GLState {
    pub(crate) m_capabilities: GLCapabilities,
    pub(crate) m_scissorBox: [u32; 4],
    pub(crate) m_scissorEnabled: bool,
    pub(crate) m_depthTestEnabled: bool,
    pub(crate) m_stencilTestEnabled: bool,
    pub(crate) m_colorWriteMask: bool,
    pub(crate) m_depthWriteMask: bool,
    pub(crate) m_blendEquation: BlendEquation,
    pub(crate) m_stencilWriteMask: GLuint,
    pub(crate) m_cullFace: GLenum,
    pub(crate) m_boundProgramID: GLuint,
    pub(crate) m_boundVAO: GLuint,
    pub(crate) m_boundArrayBufferID: GLuint,
    pub(crate) m_boundUniformBufferID: GLuint,
    pub(crate) m_validState: ValidState,
}

impl GLState {
    pub(crate) fn new(capabilities: GLCapabilities) -> Self {
        let mut state = Self {
            m_capabilities: capabilities,
            m_scissorBox: [0; 4],
            m_scissorEnabled: false,
            m_depthTestEnabled: false,
            m_stencilTestEnabled: false,
            m_colorWriteMask: false,
            m_depthWriteMask: false,
            m_blendEquation: BlendEquation::none,
            m_stencilWriteMask: 0,
            m_cullFace: 0,
            m_boundProgramID: 0,
            m_boundVAO: 0,
            m_boundArrayBufferID: 0,
            m_boundUniformBufferID: 0,
            m_validState: ValidState::default(),
        };
        state.invalidate();
        state
    }

    pub(crate) fn capabilities(&self) -> &GLCapabilities {
        &self.m_capabilities
    }

    pub(crate) fn invalidate(&mut self) {
        super::gl_state_impl::invalidate(self)
    }

    pub(crate) fn setScissor(&mut self, scissor: IAABB, renderTargetHeight: u32) {
        super::gl_state_impl::setScissor(self, scissor, renderTargetHeight)
    }

    pub(crate) fn setScissorU16(&mut self, scissor: AABBu16, renderTargetHeight: u32) {
        super::gl_state_impl::setScissorU16(self, scissor, renderTargetHeight)
    }

    pub(crate) fn setScissorRaw(&mut self, left: u32, top: u32, width: u32, height: u32) {
        super::gl_state_impl::setScissorRaw(self, left, top, width, height)
    }

    pub(crate) fn disableScissor(&mut self) {
        super::gl_state_impl::disableScissor(self)
    }

    pub(crate) fn setDepthStencilEnabled(&mut self, depthEnabled: bool, stencilEnabled: bool) {
        super::gl_state_impl::setDepthStencilEnabled(self, depthEnabled, stencilEnabled)
    }

    pub(crate) fn setCullFace(&mut self, cullFace: GLenum) {
        super::gl_state_impl::setCullFace(self, cullFace)
    }

    pub(crate) fn setWriteMasks(
        &mut self,
        colorWriteMask: bool,
        depthWriteMask: bool,
        stencilWriteMask: u8,
    ) {
        super::gl_state_impl::setWriteMasks(
            self,
            colorWriteMask,
            depthWriteMask,
            stencilWriteMask,
        )
    }

    pub(crate) fn setBlendEquation(&mut self, blendEquation: BlendEquation) {
        super::gl_state_impl::setBlendEquation(self, blendEquation)
    }

    pub(crate) fn disableBlending(&mut self) {
        self.setBlendEquation(BlendEquation::none)
    }

    pub(crate) fn setPipelineState(
        &mut self,
        pipelineState: &PipelineState,
        scissorAction: ScissorAction,
    ) {
        super::gl_state_impl::setPipelineState(self, pipelineState, scissorAction)
    }

    pub(crate) fn bindProgram(&mut self, programID: GLuint) {
        super::gl_state_impl::bindProgram(self, programID)
    }

    pub(crate) fn bindVAO(&mut self, vao: GLuint) {
        super::gl_state_impl::bindVAO(self, vao)
    }

    pub(crate) fn bindBuffer(&mut self, target: GLenum, bufferID: GLuint) {
        super::gl_state_impl::bindBuffer(self, target, bufferID)
    }

    pub(crate) fn deleteProgram(&mut self, programID: GLuint) {
        super::gl_state_impl::deleteProgram(self, programID)
    }

    pub(crate) fn deleteVAO(&mut self, vao: GLuint) {
        super::gl_state_impl::deleteVAO(self, vao)
    }

    pub(crate) fn deleteBuffer(&mut self, bufferID: GLuint) {
        super::gl_state_impl::deleteBuffer(self, bufferID)
    }
}
