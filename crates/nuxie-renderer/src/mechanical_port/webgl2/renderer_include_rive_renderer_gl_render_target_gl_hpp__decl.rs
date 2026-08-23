//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/gl/render_target_gl.hpp` for `RIVE_WEBGL`.

#![allow(non_snake_case)]

use super::gl_utils_decl::{Framebuffer, Renderbuffer, Texture};
use super::gles3_decl::{GLCapabilities, GLExecutionStamp, GLenum, GLuint};
use super::render_context_gl_decl::RenderContextGLImpl;
use crate::mechanical_port::source::include::rive::refcnt_hpp::RefCntTarget;
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{
    enable_lite_rtti, LiteRttiBase, LiteRttiCastFrom, LiteRttiTypeId, CONST_ID,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::IAABB;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_include_rive_renderer_gl_render_target_gl.hpp");

pub(crate) const RENDER_TARGET_GL_LITE_RTTI_TYPE_ID: u32 = CONST_ID("RenderTargetGL");
pub(crate) const TEXTURE_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID: u32 =
    CONST_ID("TextureRenderTargetGL");
pub(crate) const FRAMEBUFFER_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID: u32 =
    CONST_ID("FramebufferRenderTargetGL");

pub(crate) type RenderTargetGLEnableLiteRtti =
    enable_lite_rtti<RenderTargetGL, { RENDER_TARGET_GL_LITE_RTTI_TYPE_ID }>;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MSAAResolveAction {
    automatic = 0,
    framebufferBlit = 1,
}

/// The source's pure-virtual `RenderTargetGL` interface. The complete concrete
/// owner remains either `TextureRenderTargetGL` or
/// `FramebufferRenderTargetGL`; this trait does not collapse that distinction.
pub(crate) trait RenderTargetGLApi {
    fn base(&self) -> &RenderTargetGL;
    fn baseMut(&mut self) -> &mut RenderTargetGL;

    fn bindDestinationFramebuffer(&mut self, target: GLenum);
    fn renderTexture(&mut self) -> GLuint;
    fn bindTextureFramebuffer(&mut self, target: GLenum);
    fn bindHeadlessFramebuffer(&mut self, capabilities: &GLCapabilities);
    fn bindMSAAFramebuffer(
        &mut self,
        renderContextImpl: &mut RenderContextGLImpl,
        sampleCount: i32,
        preserveBounds: Option<&IAABB>,
        isFBO0: Option<&mut bool>,
    ) -> MSAAResolveAction;
    fn allocateWebGLPLSBacking(&mut self, capabilities: &GLCapabilities);
}

#[repr(C)]
pub(crate) struct RenderTargetGL {
    // public RenderTarget base, followed by ENABLE_LITE_RTTI(RenderTargetGL).
    pub(crate) base: ManuallyDrop<RenderTarget>,
    pub(crate) lite_rtti: ManuallyDrop<RenderTargetGLEnableLiteRtti>,

    pub(crate) m_dstColorTexture: ManuallyDrop<Texture>,
    pub(crate) m_dstColorFramebuffer: ManuallyDrop<Framebuffer>,

    /// Rust-only creation identity after the complete RenderTargetGL source
    /// field prefix.
    pub(crate) rust_execution: ManuallyDrop<GLExecutionStamp>,
}

impl RenderTargetGL {
    pub(crate) fn newBase(width: u32, height: u32, execution: GLExecutionStamp) -> Self {
        let mut base = RenderTarget::new(width, height);
        base.install_owner_thread_execution(
            execution.domain().ownerThreadFinalReleaseRoute(),
            execution.domain().key(),
            execution.generation(),
        );
        Self {
            base: ManuallyDrop::new(base),
            lite_rtti: ManuallyDrop::new(RenderTargetGLEnableLiteRtti::new()),
            m_dstColorTexture: ManuallyDrop::new(Texture::Zero()),
            m_dstColorFramebuffer: ManuallyDrop::new(Framebuffer::Zero()),
            rust_execution: ManuallyDrop::new(execution),
        }
    }

    pub(crate) fn executionStamp(&self) -> &GLExecutionStamp {
        &self.rust_execution
    }

    pub(crate) fn assertSameExecution(&self, execution: &GLExecutionStamp) {
        assert!(
            self.executionStamp().sameDomain(execution),
            "RenderTargetGL and RenderContextGLImpl belong to different context identities or generations"
        );
    }

    pub(crate) fn width(&self) -> u32 {
        self.base.width()
    }

    pub(crate) fn height(&self) -> u32 {
        self.base.height()
    }

    pub(crate) fn dstColorTexture(&mut self) -> GLuint {
        super::render_target_gl_impl::dstColorTexture(self)
    }

    pub(crate) fn bindDstColorFramebuffer(&mut self, target: GLenum) {
        super::render_target_gl_impl::bindDstColorFramebuffer(self, target)
    }
}

impl LiteRttiBase for RenderTargetGL {
    fn liteTypeID(&self) -> u32 {
        self.lite_rtti.liteTypeID()
    }

    fn setLiteTypeID(&mut self, id: u32) {
        self.lite_rtti.setLiteTypeID(id);
    }
}

#[repr(C)]
pub(crate) struct TextureRenderTargetGL {
    pub(crate) base: ManuallyDrop<RenderTargetGL>,

    // Not owned or deleted by this target.
    pub(crate) m_externalTextureID: GLuint,

    pub(crate) m_framebufferID: ManuallyDrop<Framebuffer>,
    pub(crate) m_headlessFramebuffer: ManuallyDrop<Framebuffer>,
    pub(crate) m_framebufferTargetAttachmentDirty: bool,

    pub(crate) m_webglPLSBackingR32UI: ManuallyDrop<Texture>,
    pub(crate) m_webglPLSBackingR32UIFallback: ManuallyDrop<Texture>,
    pub(crate) m_webglPLSBackingRGBA8: ManuallyDrop<Texture>,
    pub(crate) m_webglPLSBindingsDirty: bool,

    pub(crate) m_msaaFramebuffer: ManuallyDrop<Framebuffer>,
    pub(crate) m_msaaColorBuffer: ManuallyDrop<Renderbuffer>,
    pub(crate) m_msaaDepthStencilBuffer: ManuallyDrop<Renderbuffer>,
    pub(crate) m_msaaFramebufferSampleCount: i32,
}

impl TextureRenderTargetGL {
    pub(crate) fn new(width: u32, height: u32, execution: GLExecutionStamp) -> Self {
        super::render_target_gl_impl::newTextureRenderTargetGL(width, height, execution)
    }

    pub(crate) fn externalTextureID(&self) -> GLuint {
        self.m_externalTextureID
    }

    pub(crate) fn setTargetTexture(&mut self, externalTextureID: GLuint) {
        self.m_externalTextureID = externalTextureID;
        self.m_framebufferTargetAttachmentDirty = true;
        self.m_webglPLSBindingsDirty = true;
    }

    pub(crate) fn bindDestinationFramebuffer(&mut self, target: GLenum) {
        self.bindTextureFramebuffer(target);
    }

    pub(crate) fn renderTexture(&self) -> GLuint {
        self.externalTextureID()
    }

    pub(crate) fn bindTextureFramebuffer(&mut self, target: GLenum) {
        super::render_target_gl_impl::bindTextureFramebuffer(self, target)
    }

    pub(crate) fn bindHeadlessFramebuffer(&mut self, capabilities: &GLCapabilities) {
        super::render_target_gl_impl::bindHeadlessFramebuffer(self, capabilities)
    }

    pub(crate) fn bindMSAAFramebuffer(
        &mut self,
        renderContextImpl: &mut RenderContextGLImpl,
        sampleCount: i32,
        preserveBounds: Option<&IAABB>,
        isFBO0: Option<&mut bool>,
    ) -> MSAAResolveAction {
        super::render_target_gl_impl::bindTextureMSAAFramebuffer(
            self,
            renderContextImpl,
            sampleCount,
            preserveBounds,
            isFBO0,
        )
    }

    pub(crate) fn allocateWebGLPLSBacking(&mut self, capabilities: &GLCapabilities) {
        super::render_target_gl_impl::allocateTextureWebGLPLSBacking(self, capabilities)
    }
}

impl Deref for TextureRenderTargetGL {
    type Target = RenderTargetGL;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for TextureRenderTargetGL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl LiteRttiTypeId for TextureRenderTargetGL {
    const LITE_RTTI_TYPE_ID: u32 = TEXTURE_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID;
}

impl LiteRttiCastFrom<RenderTargetGL> for TextureRenderTargetGL {
    unsafe fn from_base(base: *mut RenderTargetGL) -> *mut Self {
        base.cast()
    }
}

unsafe impl RefCntTarget for TextureRenderTargetGL {
    fn r#ref(&self) {
        self.base.base.r#ref();
    }

    unsafe fn unref(&self) {
        unsafe { self.base.base.unref() };
    }

    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { drop(Box::from_raw(ptr.cast_mut())) };
    }
}

impl RenderTargetGLApi for TextureRenderTargetGL {
    fn base(&self) -> &RenderTargetGL {
        &self.base
    }

    fn baseMut(&mut self) -> &mut RenderTargetGL {
        &mut self.base
    }

    fn bindDestinationFramebuffer(&mut self, target: GLenum) {
        TextureRenderTargetGL::bindDestinationFramebuffer(self, target)
    }

    fn renderTexture(&mut self) -> GLuint {
        TextureRenderTargetGL::renderTexture(self)
    }

    fn bindTextureFramebuffer(&mut self, target: GLenum) {
        TextureRenderTargetGL::bindTextureFramebuffer(self, target)
    }

    fn bindHeadlessFramebuffer(&mut self, capabilities: &GLCapabilities) {
        TextureRenderTargetGL::bindHeadlessFramebuffer(self, capabilities)
    }

    fn bindMSAAFramebuffer(
        &mut self,
        renderContextImpl: &mut RenderContextGLImpl,
        sampleCount: i32,
        preserveBounds: Option<&IAABB>,
        isFBO0: Option<&mut bool>,
    ) -> MSAAResolveAction {
        TextureRenderTargetGL::bindMSAAFramebuffer(
            self,
            renderContextImpl,
            sampleCount,
            preserveBounds,
            isFBO0,
        )
    }

    fn allocateWebGLPLSBacking(&mut self, capabilities: &GLCapabilities) {
        TextureRenderTargetGL::allocateWebGLPLSBacking(self, capabilities)
    }
}

#[repr(C)]
pub(crate) struct FramebufferRenderTargetGL {
    pub(crate) base: ManuallyDrop<RenderTargetGL>,

    // Both client-provided values remain immutable and the framebuffer remains
    // borrowed for the complete owner lifetime.
    pub(crate) m_externalFramebufferID: GLuint,
    pub(crate) m_sampleCount: u32,

    pub(crate) m_textureRenderTarget: ManuallyDrop<TextureRenderTargetGL>,
    pub(crate) m_offscreenTargetTexture: ManuallyDrop<Texture>,
}

impl FramebufferRenderTargetGL {
    pub(crate) fn new(
        width: u32,
        height: u32,
        externalFramebufferID: GLuint,
        sampleCount: u32,
        execution: GLExecutionStamp,
    ) -> Self {
        super::render_target_gl_impl::newFramebufferRenderTargetGL(
            width,
            height,
            externalFramebufferID,
            sampleCount,
            execution,
        )
    }

    pub(crate) fn sampleCount(&self) -> u32 {
        self.m_sampleCount
    }

    pub(crate) fn allocateOffscreenTargetTexture(&mut self) {
        super::render_target_gl_impl::allocateOffscreenTargetTexture(self)
    }

    pub(crate) fn bindDestinationFramebuffer(&mut self, target: GLenum) {
        super::render_target_gl_impl::bindFramebufferDestinationFramebuffer(self, target)
    }

    pub(crate) fn renderTexture(&mut self) -> GLuint {
        super::render_target_gl_impl::framebufferRenderTexture(self)
    }

    pub(crate) fn bindTextureFramebuffer(&mut self, target: GLenum) {
        super::render_target_gl_impl::bindFramebufferTextureFramebuffer(self, target)
    }

    pub(crate) fn bindHeadlessFramebuffer(&mut self, capabilities: &GLCapabilities) {
        super::render_target_gl_impl::bindFramebufferHeadlessFramebuffer(self, capabilities)
    }

    pub(crate) fn bindMSAAFramebuffer(
        &mut self,
        renderContextImpl: &mut RenderContextGLImpl,
        sampleCount: i32,
        preserveBounds: Option<&IAABB>,
        isFBO0: Option<&mut bool>,
    ) -> MSAAResolveAction {
        super::render_target_gl_impl::bindFramebufferMSAAFramebuffer(
            self,
            renderContextImpl,
            sampleCount,
            preserveBounds,
            isFBO0,
        )
    }

    pub(crate) fn allocateWebGLPLSBacking(&mut self, capabilities: &GLCapabilities) {
        super::render_target_gl_impl::allocateFramebufferWebGLPLSBacking(self, capabilities)
    }
}

impl Deref for FramebufferRenderTargetGL {
    type Target = RenderTargetGL;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for FramebufferRenderTargetGL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl LiteRttiTypeId for FramebufferRenderTargetGL {
    const LITE_RTTI_TYPE_ID: u32 = FRAMEBUFFER_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID;
}

impl LiteRttiCastFrom<RenderTargetGL> for FramebufferRenderTargetGL {
    unsafe fn from_base(base: *mut RenderTargetGL) -> *mut Self {
        base.cast()
    }
}

unsafe impl RefCntTarget for FramebufferRenderTargetGL {
    fn r#ref(&self) {
        self.base.base.r#ref();
    }

    unsafe fn unref(&self) {
        unsafe { self.base.base.unref() };
    }

    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { drop(Box::from_raw(ptr.cast_mut())) };
    }
}

impl RenderTargetGLApi for FramebufferRenderTargetGL {
    fn base(&self) -> &RenderTargetGL {
        &self.base
    }

    fn baseMut(&mut self) -> &mut RenderTargetGL {
        &mut self.base
    }

    fn bindDestinationFramebuffer(&mut self, target: GLenum) {
        FramebufferRenderTargetGL::bindDestinationFramebuffer(self, target)
    }

    fn renderTexture(&mut self) -> GLuint {
        FramebufferRenderTargetGL::renderTexture(self)
    }

    fn bindTextureFramebuffer(&mut self, target: GLenum) {
        FramebufferRenderTargetGL::bindTextureFramebuffer(self, target)
    }

    fn bindHeadlessFramebuffer(&mut self, capabilities: &GLCapabilities) {
        FramebufferRenderTargetGL::bindHeadlessFramebuffer(self, capabilities)
    }

    fn bindMSAAFramebuffer(
        &mut self,
        renderContextImpl: &mut RenderContextGLImpl,
        sampleCount: i32,
        preserveBounds: Option<&IAABB>,
        isFBO0: Option<&mut bool>,
    ) -> MSAAResolveAction {
        FramebufferRenderTargetGL::bindMSAAFramebuffer(
            self,
            renderContextImpl,
            sampleCount,
            preserveBounds,
            isFBO0,
        )
    }

    fn allocateWebGLPLSBacking(&mut self, capabilities: &GLCapabilities) {
        FramebufferRenderTargetGL::allocateWebGLPLSBacking(self, capabilities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::offset_of;

    #[test]
    fn complete_header_denominator_and_execution_sidecar_are_frozen() {
        assert_eq!(PINNED_SOURCE.lines().count(), 210);
        assert_eq!(offset_of!(RenderTargetGL, base), 0);
        assert!(offset_of!(RenderTargetGL, lite_rtti) > offset_of!(RenderTargetGL, base));
        assert!(
            offset_of!(RenderTargetGL, m_dstColorTexture) > offset_of!(RenderTargetGL, lite_rtti)
        );
        assert!(
            offset_of!(RenderTargetGL, m_dstColorFramebuffer)
                > offset_of!(RenderTargetGL, m_dstColorTexture)
        );
        assert!(
            offset_of!(RenderTargetGL, rust_execution)
                > offset_of!(RenderTargetGL, m_dstColorFramebuffer)
        );

        assert_eq!(offset_of!(TextureRenderTargetGL, base), 0);
        assert!(
            offset_of!(TextureRenderTargetGL, m_externalTextureID)
                > offset_of!(TextureRenderTargetGL, base)
        );
        assert!(
            offset_of!(TextureRenderTargetGL, m_msaaFramebufferSampleCount)
                > offset_of!(TextureRenderTargetGL, m_msaaDepthStencilBuffer)
        );

        assert_eq!(offset_of!(FramebufferRenderTargetGL, base), 0);
        assert!(
            offset_of!(FramebufferRenderTargetGL, m_externalFramebufferID)
                > offset_of!(FramebufferRenderTargetGL, base)
        );
        assert!(
            offset_of!(FramebufferRenderTargetGL, m_offscreenTargetTexture)
                > offset_of!(FramebufferRenderTargetGL, m_textureRenderTarget)
        );
    }
}
