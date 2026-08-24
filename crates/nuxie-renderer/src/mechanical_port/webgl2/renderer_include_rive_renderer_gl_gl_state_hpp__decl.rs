//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/gl/gl_state.hpp`.

#![allow(non_camel_case_types, non_snake_case)]

use super::gles3_decl::{GLCapabilities, GLExecutionDomain, GLExecutionStamp, GLenum, GLuint};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    AABBu16, BlendEquation, PipelineState, IAABB,
};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_include_rive_renderer_gl_gl_state.hpp");

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScissorAction {
    disable = 0,
    ignore = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ValidState {
    bits: [u8; 2],
}

macro_rules! valid_state_bit {
    ($get:ident, $set:ident, $bit:expr) => {
        pub(crate) const fn $get(&self) -> bool {
            self.bits[$bit / 8] & (1 << ($bit % 8)) != 0
        }
        pub(crate) fn $set(&mut self, value: bool) {
            let byte = $bit / 8;
            let mask = 1 << ($bit % 8);
            self.bits[byte] = (self.bits[byte] & !mask) | ((value as u8) * mask);
        }
    };
}

impl ValidState {
    valid_state_bit!(scissorBox, setScissorBox, 0);
    valid_state_bit!(scissorEnabled, setScissorEnabled, 1);
    valid_state_bit!(depthStencilEnabled, setDepthStencilEnabled, 2);
    valid_state_bit!(writeMasks, setWriteMasks, 3);
    valid_state_bit!(blendEquation, setBlendEquation, 4);
    valid_state_bit!(cullFace, setCullFace, 5);
    valid_state_bit!(boundProgramID, setBoundProgramID, 6);
    valid_state_bit!(boundVAO, setBoundVAO, 7);
    valid_state_bit!(boundArrayBufferID, setBoundArrayBufferID, 8);
    valid_state_bit!(boundUniformBufferID, setBoundUniformBufferID, 9);
    valid_state_bit!(boundPixelUnpackBufferID, setBoundPixelUnpackBufferID, 10);
}

const _: () = assert!(core::mem::size_of::<ValidState>() == 2);
const _: () = assert!(core::mem::align_of::<ValidState>() == 1);

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
    /// Rust-only retained execution authority after the exact source prefix.
    pub(crate) m_executionDomain: Option<GLExecutionDomain>,
    pub(crate) m_executionGeneration: u64,
}

impl GLState {
    pub(crate) fn new(capabilities: GLCapabilities) -> Self {
        Self::newWithExecutionDomain(capabilities, None)
    }

    pub(crate) fn newInDomain(
        capabilities: GLCapabilities,
        executionDomain: GLExecutionDomain,
    ) -> Self {
        Self::newWithExecutionDomain(capabilities, Some(executionDomain))
    }

    fn newWithExecutionDomain(
        capabilities: GLCapabilities,
        executionDomain: Option<GLExecutionDomain>,
    ) -> Self {
        let executionGeneration = executionDomain
            .as_ref()
            .map_or(0, GLExecutionDomain::generation);
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
            m_executionDomain: executionDomain,
            m_executionGeneration: executionGeneration,
        };
        state.invalidate();
        state
    }

    pub(crate) fn capabilities(&self) -> &GLCapabilities {
        &self.m_capabilities
    }

    pub(crate) fn executionDomain(&self) -> Option<GLExecutionDomain> {
        self.m_executionDomain.clone()
    }

    pub(crate) fn executionStamp(&self) -> Option<GLExecutionStamp> {
        self.m_executionDomain.as_ref().map(|executionDomain| {
            assert_eq!(
                self.m_executionGeneration,
                executionDomain.generation(),
                "GLState belongs to a stale WebGL context generation"
            );
            executionDomain.stamp()
        })
    }

    fn withExecutionDomain<R>(&mut self, callback: impl FnOnce(&mut Self) -> R) -> R {
        if let Some(executionDomain) = self.m_executionDomain.clone() {
            assert_eq!(
                self.m_executionGeneration,
                executionDomain.generation(),
                "GLState belongs to a stale WebGL context generation"
            );
            // `self` is commonly held through `RefCell::borrow_mut()` here.
            // Enter the GL domain without treating this live source borrow as
            // a final-release safe point: a queued RenderBuffer destructor can
            // otherwise reborrow the same GLState and panic.
            executionDomain.withCurrentWhileSourceBorrowed(|| callback(self))
        } else {
            callback(self)
        }
    }

    pub(crate) fn invalidate(&mut self) {
        self.withExecutionDomain(super::gl_state_impl::invalidate)
    }

    pub(crate) fn setScissor(&mut self, scissor: IAABB, renderTargetHeight: u32) {
        self.withExecutionDomain(|state| {
            super::gl_state_impl::setScissor(state, scissor, renderTargetHeight)
        })
    }

    pub(crate) fn setScissorU16(&mut self, scissor: AABBu16, renderTargetHeight: u32) {
        self.withExecutionDomain(|state| {
            super::gl_state_impl::setScissorU16(state, scissor, renderTargetHeight)
        })
    }

    pub(crate) fn setScissorRaw(&mut self, left: u32, top: u32, width: u32, height: u32) {
        self.withExecutionDomain(|state| {
            super::gl_state_impl::setScissorRaw(state, left, top, width, height)
        })
    }

    pub(crate) fn disableScissor(&mut self) {
        self.withExecutionDomain(super::gl_state_impl::disableScissor)
    }

    pub(crate) fn setDepthStencilEnabled(&mut self, depthEnabled: bool, stencilEnabled: bool) {
        self.withExecutionDomain(|state| {
            super::gl_state_impl::setDepthStencilEnabled(state, depthEnabled, stencilEnabled)
        })
    }

    pub(crate) fn setCullFace(&mut self, cullFace: GLenum) {
        self.withExecutionDomain(|state| super::gl_state_impl::setCullFace(state, cullFace))
    }

    pub(crate) fn setWriteMasks(
        &mut self,
        colorWriteMask: bool,
        depthWriteMask: bool,
        stencilWriteMask: u8,
    ) {
        self.withExecutionDomain(|state| {
            super::gl_state_impl::setWriteMasks(
                state,
                colorWriteMask,
                depthWriteMask,
                stencilWriteMask,
            )
        })
    }

    pub(crate) fn setBlendEquation(&mut self, blendEquation: BlendEquation) {
        self.withExecutionDomain(|state| {
            super::gl_state_impl::setBlendEquation(state, blendEquation)
        })
    }

    pub(crate) fn disableBlending(&mut self) {
        self.setBlendEquation(BlendEquation::none)
    }

    pub(crate) fn setPipelineState(
        &mut self,
        pipelineState: &PipelineState,
        scissorAction: ScissorAction,
    ) {
        self.withExecutionDomain(|state| {
            super::gl_state_impl::setPipelineState(state, pipelineState, scissorAction)
        })
    }

    pub(crate) fn bindProgram(&mut self, programID: GLuint) {
        self.withExecutionDomain(|state| super::gl_state_impl::bindProgram(state, programID))
    }

    pub(crate) fn bindVAO(&mut self, vao: GLuint) {
        self.withExecutionDomain(|state| super::gl_state_impl::bindVAO(state, vao))
    }

    pub(crate) fn bindBuffer(&mut self, target: GLenum, bufferID: GLuint) {
        self.withExecutionDomain(|state| super::gl_state_impl::bindBuffer(state, target, bufferID))
    }

    pub(crate) fn deleteProgram(&mut self, programID: GLuint) {
        if self
            .m_executionDomain
            .as_ref()
            .is_some_and(|domain| self.m_executionGeneration != domain.generation())
        {
            return;
        }
        self.withExecutionDomain(|state| super::gl_state_impl::deleteProgram(state, programID))
    }

    pub(crate) fn deleteVAO(&mut self, vao: GLuint) {
        if self
            .m_executionDomain
            .as_ref()
            .is_some_and(|domain| self.m_executionGeneration != domain.generation())
        {
            return;
        }
        self.withExecutionDomain(|state| super::gl_state_impl::deleteVAO(state, vao))
    }

    pub(crate) fn deleteBuffer(&mut self, bufferID: GLuint) {
        if self
            .m_executionDomain
            .as_ref()
            .is_some_and(|domain| self.m_executionGeneration != domain.generation())
        {
            return;
        }
        self.withExecutionDomain(|state| super::gl_state_impl::deleteBuffer(state, bufferID))
    }
}
