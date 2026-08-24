//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/gl/gl_utils.hpp`.

#![allow(non_camel_case_types, non_snake_case)]

use super::gles3_decl::{
    stampCurrentGLObject, GLCapabilities, GLExecutionStamp, GLenum, GLuint,
};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_include_rive_renderer_gl_gl_utils.hpp");

// The leading underscore is source-significant: generated minification never
// collides with this fallback uniform name.
pub(crate) const BASE_INSTANCE_UNIFORM_NAME: &str = "_baseInstance";

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DebugPrintErrorAndAbort {
    no = 0,
    #[default]
    yes = 1,
}

/// The two preprocessor authorities in gl_utils.cpp. They remain explicit so
/// neither the DEBUG validation path nor the Emscripten-parser bypass can turn
/// into an implicit fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GLUtilsSourceConfiguration {
    pub(crate) debug: bool,
    pub(crate) bypassEmscriptenShaderParser: bool,
}

impl GLUtilsSourceConfiguration {
    pub(crate) const FROZEN_WEBGL2: Self = Self {
        debug: cfg!(debug_assertions),
        bypassEmscriptenShaderParser: false,
    };
}

#[derive(Debug, Default)]
pub(crate) struct GLObject {
    m_id: GLuint,
    executionStamp: Option<GLExecutionStamp>,
}

impl GLObject {
    pub(crate) fn fromAdoptedID(adoptedID: GLuint) -> Self {
        Self {
            m_id: adoptedID,
            executionStamp: stampCurrentGLObject(adoptedID),
        }
    }

    pub(crate) fn fromAdoptedIDInExecution(
        adoptedID: GLuint,
        executionStamp: GLExecutionStamp,
    ) -> Self {
        assert!(
            adoptedID == 0 || executionStamp.isCurrentGeneration(),
            "a GL name can only be adopted into its live creation generation"
        );
        Self {
            m_id: adoptedID,
            executionStamp: (adoptedID != 0).then_some(executionStamp),
        }
    }

    pub(crate) const fn Zero() -> Self {
        Self {
            m_id: 0,
            executionStamp: None,
        }
    }

    pub(crate) const fn id(&self) -> GLuint {
        self.m_id
    }

    pub(crate) fn replaceWithAdoptedID(&mut self, adoptedID: GLuint) -> Self {
        std::mem::replace(self, Self::fromAdoptedID(adoptedID))
    }

    pub(crate) fn replaceWithObject(&mut self, object: Self) -> Self {
        std::mem::replace(self, object)
    }

    pub(crate) fn takeObject(&mut self) -> Self {
        std::mem::replace(self, Self::Zero())
    }

    #[cfg(test)]
    pub(crate) fn setSyntheticID(&mut self, id: GLuint) {
        assert!(self.executionStamp.is_none());
        self.m_id = id;
    }

    pub(crate) fn withDeleteCurrent(&self, callback: impl FnOnce()) {
        if let Some(stamp) = self.executionStamp.as_ref() {
            let _ = stamp.withDeleteCurrent(callback);
            return;
        }

        #[cfg(test)]
        callback();

        #[cfg(not(test))]
        debug_assert_eq!(self.m_id, 0, "nonzero GL objects always carry an execution stamp");
    }
}

impl PartialEq for GLObject {
    fn eq(&self, other: &Self) -> bool {
        self.m_id == other.m_id
    }
}

impl Eq for GLObject {}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Buffer(pub(crate) GLObject);

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Texture(pub(crate) GLObject);

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Framebuffer(pub(crate) GLObject);

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Renderbuffer(pub(crate) GLObject);

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct VAO(pub(crate) GLObject);

#[repr(transparent)]
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Shader(pub(crate) GLObject);

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Program {
    pub(crate) m_object: GLObject,
    pub(crate) m_vertexShader: Shader,
    pub(crate) m_fragmentShader: Shader,
}

impl Buffer {
    pub(crate) fn new() -> Self {
        super::gl_utils_impl::newBuffer()
    }

    pub(crate) const fn id(&self) -> GLuint {
        self.0.id()
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Texture {
    pub(crate) fn new() -> Self {
        super::gl_utils_impl::newTexture()
    }

    pub(crate) const fn Zero() -> Self {
        Self(GLObject::Zero())
    }

    pub(crate) fn Adopt(id: GLuint) -> Self {
        Self(GLObject::fromAdoptedID(id))
    }

    pub(crate) fn AdoptInExecution(id: GLuint, executionStamp: GLExecutionStamp) -> Self {
        Self(GLObject::fromAdoptedIDInExecution(id, executionStamp))
    }

    pub(crate) const fn id(&self) -> GLuint {
        self.0.id()
    }

    pub(crate) fn moveAssign(&mut self, rhs: Self) {
        super::gl_utils_impl::moveAssignTexture(self, rhs)
    }

    pub(crate) fn reset(&mut self, adoptedID: GLuint) {
        super::gl_utils_impl::resetTexture(self, adoptedID)
    }
}

impl Default for Texture {
    fn default() -> Self {
        Self::new()
    }
}

impl Framebuffer {
    pub(crate) fn new() -> Self {
        super::gl_utils_impl::newFramebuffer()
    }

    pub(crate) const fn Zero() -> Self {
        Self(GLObject::Zero())
    }

    pub(crate) const fn id(&self) -> GLuint {
        self.0.id()
    }

    pub(crate) fn moveAssign(&mut self, rhs: Self) {
        super::gl_utils_impl::moveAssignFramebuffer(self, rhs)
    }

    pub(crate) fn reset(&mut self, adoptedID: GLuint) {
        super::gl_utils_impl::resetFramebuffer(self, adoptedID)
    }
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderbuffer {
    pub(crate) fn new() -> Self {
        super::gl_utils_impl::newRenderbuffer()
    }

    pub(crate) const fn Zero() -> Self {
        Self(GLObject::Zero())
    }

    pub(crate) const fn id(&self) -> GLuint {
        self.0.id()
    }

    pub(crate) fn moveAssign(&mut self, rhs: Self) {
        super::gl_utils_impl::moveAssignRenderbuffer(self, rhs)
    }

    pub(crate) fn reset(&mut self, adoptedID: GLuint) {
        super::gl_utils_impl::resetRenderbuffer(self, adoptedID)
    }
}

impl Default for Renderbuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl VAO {
    pub(crate) fn new() -> Self {
        super::gl_utils_impl::newVAO()
    }

    pub(crate) const fn id(&self) -> GLuint {
        self.0.id()
    }
}

impl Default for VAO {
    fn default() -> Self {
        Self::new()
    }
}

impl Shader {
    pub(crate) const fn id(&self) -> GLuint {
        self.0.id()
    }

    pub(crate) fn compile(
        &mut self,
        shaderType: GLenum,
        source: &str,
        capabilities: &GLCapabilities,
    ) {
        self.compileParts(shaderType, &[], &[source], capabilities)
    }

    pub(crate) fn compileParts(
        &mut self,
        shaderType: GLenum,
        defines: &[&str],
        sources: &[&str],
        capabilities: &GLCapabilities,
    ) {
        super::gl_utils_impl::compileOwnedShader(
            self,
            shaderType,
            defines,
            sources,
            capabilities,
        )
    }

    pub(crate) fn reset(&mut self, adoptedID: GLuint) {
        super::gl_utils_impl::resetShader(self, adoptedID)
    }
}

impl Program {
    pub(crate) fn new() -> Self {
        super::gl_utils_impl::newProgram()
    }

    pub(crate) const fn Zero() -> Self {
        Self {
            m_object: GLObject::Zero(),
            m_vertexShader: Shader(GLObject::Zero()),
            m_fragmentShader: Shader(GLObject::Zero()),
        }
    }

    pub(crate) const fn id(&self) -> GLuint {
        self.m_object.id()
    }

    pub(crate) fn moveAssign(&mut self, rhs: Self) {
        super::gl_utils_impl::moveAssignProgram(self, rhs)
    }

    pub(crate) fn compileAndAttachShader(
        &mut self,
        shaderType: GLenum,
        source: &str,
        capabilities: &GLCapabilities,
    ) {
        self.compileAndAttachShaderParts(shaderType, &[], &[source], capabilities)
    }

    pub(crate) fn compileAndAttachShaderParts(
        &mut self,
        shaderType: GLenum,
        defines: &[&str],
        sources: &[&str],
        capabilities: &GLCapabilities,
    ) {
        super::gl_utils_impl::compileAndAttachOwnedShader(
            self,
            shaderType,
            defines,
            sources,
            capabilities,
        )
    }

    pub(crate) fn link(&self) {
        super::gl_utils_impl::LinkProgram(self.id(), DebugPrintErrorAndAbort::yes)
    }

    pub(crate) fn reset(&mut self, adoptedProgramID: GLuint) {
        super::gl_utils_impl::resetProgram(self, adoptedProgramID)
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) use super::gl_utils_impl::{
    BlitFramebuffer, CompileAndAttachShader, CompileAndAttachShaderParts, CompileRawGLSL,
    CompileRawGLSLWithConfiguration, CompileShader, CompileShaderParts, LinkProgram,
    LinkProgramWithConfiguration, PrintLinkProgramErrors, PrintShaderCompilationErrors,
    SetTexture2DSamplingParams, SetTexture2DSamplingParamsFromSampler, Uniform1iByName,
};
