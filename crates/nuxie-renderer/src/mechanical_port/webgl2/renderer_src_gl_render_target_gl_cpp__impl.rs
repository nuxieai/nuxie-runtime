//! Complete mechanical implementation translation of
//! `renderer/src/gl/render_target_gl.cpp` for `RIVE_WEBGL`.

#![allow(non_snake_case)]

use super::gl_state_decl::ScissorAction;
use super::gl_utils_decl::{Framebuffer, Renderbuffer, Texture};
use super::gles3_decl::*;
use super::render_context_gl_decl::RenderContextGLImpl;
use super::render_target_gl_decl::{
    FramebufferRenderTargetGL, MSAAResolveAction, RenderTargetGL, TextureRenderTargetGL,
    FRAMEBUFFER_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID, TEXTURE_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID,
};
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::LiteRttiBase;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    COLOR_ONLY_PIPELINE_STATE, IAABB,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
use std::mem::ManuallyDrop;

pub(crate) const PINNED_SOURCE: &str = include_str!("source/renderer_src_gl_render_target_gl.cpp");

// renderer/src/shaders/constants.glsl is direct frozen authority for these
// four plane indices.
const COLOR_PLANE_IDX: GLint = 0;
const CLIP_PLANE_IDX: GLint = 1;
const SCRATCH_COLOR_PLANE_IDX: GLint = 2;
const COVERAGE_PLANE_IDX: GLint = 3;

pub(crate) unsafe fn destroyTextureRenderTargetGL(ptr: *mut RenderTarget) {
    unsafe { drop(Box::from_raw(ptr.cast::<TextureRenderTargetGL>())) };
}

pub(crate) unsafe fn destroyFramebufferRenderTargetGL(ptr: *mut RenderTarget) {
    unsafe { drop(Box::from_raw(ptr.cast::<FramebufferRenderTargetGL>())) };
}

pub(crate) fn newTextureRenderTargetGL(
    width: u32,
    height: u32,
    execution: GLExecutionStamp,
) -> TextureRenderTargetGL {
    let mut base = RenderTargetGL::newBase(width, height, execution);
    base.base.destroy_complete = destroyTextureRenderTargetGL;
    base.setLiteTypeID(TEXTURE_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID);
    TextureRenderTargetGL {
        base: ManuallyDrop::new(base),
        m_externalTextureID: 0,
        m_framebufferID: ManuallyDrop::new(Framebuffer::Zero()),
        m_headlessFramebuffer: ManuallyDrop::new(Framebuffer::Zero()),
        m_framebufferTargetAttachmentDirty: false,
        m_webglPLSBackingR32UI: ManuallyDrop::new(Texture::Zero()),
        m_webglPLSBackingR32UIFallback: ManuallyDrop::new(Texture::Zero()),
        m_webglPLSBackingRGBA8: ManuallyDrop::new(Texture::Zero()),
        m_webglPLSBindingsDirty: false,
        m_msaaFramebuffer: ManuallyDrop::new(Framebuffer::Zero()),
        m_msaaColorBuffer: ManuallyDrop::new(Renderbuffer::Zero()),
        m_msaaDepthStencilBuffer: ManuallyDrop::new(Renderbuffer::Zero()),
        m_msaaFramebufferSampleCount: 0,
    }
}

pub(crate) fn newFramebufferRenderTargetGL(
    width: u32,
    height: u32,
    externalFramebufferID: GLuint,
    sampleCount: u32,
    execution: GLExecutionStamp,
) -> FramebufferRenderTargetGL {
    let mut base = RenderTargetGL::newBase(width, height, execution.clone());
    base.base.destroy_complete = destroyFramebufferRenderTargetGL;
    base.setLiteTypeID(FRAMEBUFFER_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID);
    FramebufferRenderTargetGL {
        base: ManuallyDrop::new(base),
        m_externalFramebufferID: externalFramebufferID,
        m_sampleCount: sampleCount,
        m_textureRenderTarget: ManuallyDrop::new(TextureRenderTargetGL::new(
            width, height, execution,
        )),
        m_offscreenTargetTexture: ManuallyDrop::new(Texture::Zero()),
    }
}

pub(crate) fn dstColorTexture(renderTarget: &mut RenderTargetGL) -> GLuint {
    let execution = renderTarget.executionStamp().clone();
    execution.withCurrent(|| dstColorTextureCurrent(renderTarget))
}

fn dstColorTextureCurrent(renderTarget: &mut RenderTargetGL) -> GLuint {
    if renderTarget.m_dstColorTexture.id() == 0 {
        renderTarget.m_dstColorTexture.moveAssign(Texture::new());
        recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0));
        recordGLCommand(GLCommand::BindTexture(
            GL_TEXTURE_2D,
            renderTarget.m_dstColorTexture.id(),
        ));
        recordGLCommand(GLCommand::TexStorage2D {
            target: GL_TEXTURE_2D,
            levels: 1,
            internal_format: GL_RGBA8,
            width: renderTarget.width(),
            height: renderTarget.height(),
        });
    }
    renderTarget.m_dstColorTexture.id()
}

pub(crate) fn bindDstColorFramebuffer(renderTarget: &mut RenderTargetGL, target: GLenum) {
    let execution = renderTarget.executionStamp().clone();
    execution.withCurrent(|| bindDstColorFramebufferCurrent(renderTarget, target));
}

fn bindDstColorFramebufferCurrent(renderTarget: &mut RenderTargetGL, target: GLenum) {
    if renderTarget.m_dstColorFramebuffer.id() == 0 {
        renderTarget
            .m_dstColorFramebuffer
            .moveAssign(Framebuffer::new());
        recordGLCommand(GLCommand::BindFramebuffer(
            target,
            renderTarget.m_dstColorFramebuffer.id(),
        ));
        let texture = dstColorTextureCurrent(renderTarget);
        recordGLCommand(GLCommand::FramebufferTexture2D {
            target,
            attachment: GL_COLOR_ATTACHMENT0,
            texture_target: GL_TEXTURE_2D,
            texture,
            level: 0,
        });
    } else {
        recordGLCommand(GLCommand::BindFramebuffer(
            target,
            renderTarget.m_dstColorFramebuffer.id(),
        ));
    }
}

pub(crate) fn bindTextureFramebuffer(renderTarget: &mut TextureRenderTargetGL, target: GLenum) {
    let execution = renderTarget.executionStamp().clone();
    execution.withCurrent(|| bindTextureFramebufferCurrent(renderTarget, target));
}

fn bindTextureFramebufferCurrent(renderTarget: &mut TextureRenderTargetGL, target: GLenum) {
    if renderTarget.m_framebufferID.id() == 0 {
        renderTarget.m_framebufferID.moveAssign(Framebuffer::new());
    }
    recordGLCommand(GLCommand::BindFramebuffer(
        target,
        renderTarget.m_framebufferID.id(),
    ));

    if renderTarget.m_framebufferTargetAttachmentDirty {
        recordGLCommand(GLCommand::FramebufferTexture2D {
            target,
            attachment: GL_COLOR_ATTACHMENT0 + COLOR_PLANE_IDX as GLenum,
            texture_target: GL_TEXTURE_2D,
            texture: renderTarget.m_externalTextureID,
            level: 0,
        });
        renderTarget.m_framebufferTargetAttachmentDirty = false;
    }
}

pub(crate) fn bindHeadlessFramebuffer(
    renderTarget: &mut TextureRenderTargetGL,
    capabilities: &GLCapabilities,
) {
    let execution = renderTarget.executionStamp().clone();
    execution.withCurrent(|| bindHeadlessFramebufferCurrent(renderTarget, capabilities));
}

fn bindHeadlessFramebufferCurrent(
    renderTarget: &mut TextureRenderTargetGL,
    capabilities: &GLCapabilities,
) {
    if renderTarget.m_headlessFramebuffer.id() == 0 {
        renderTarget
            .m_headlessFramebuffer
            .moveAssign(Framebuffer::new());
        recordGLCommand(GLCommand::BindFramebuffer(
            GL_DRAW_FRAMEBUFFER,
            renderTarget.m_headlessFramebuffer.id(),
        ));

        // The source's ARB_shader_image_load_store framebuffer-default-size
        // branch is excluded by RIVE_WEBGL.
        recordGLCommand(GLCommand::DrawBuffers(Vec::new()));
    } else {
        recordGLCommand(GLCommand::BindFramebuffer(
            GL_DRAW_FRAMEBUFFER,
            renderTarget.m_headlessFramebuffer.id(),
        ));
    }

    // GL_ANGLE_shader_pixel_local_storage is defined by the pinned RIVE_WEBGL
    // gles3.hpp, while actual calls remain gated by the runtime capability.
    if capabilities.ANGLE_shader_pixel_local_storage && renderTarget.m_webglPLSBindingsDirty {
        recordGLCommand(GLCommand::FramebufferTexturePixelLocalStorageANGLE {
            plane: COLOR_PLANE_IDX,
            backing_texture: renderTarget.m_externalTextureID,
            level: 0,
            layer: 0,
            usage: GL_NONE,
        });
        recordGLCommand(GLCommand::FramebufferTexturePixelLocalStorageANGLE {
            plane: COVERAGE_PLANE_IDX,
            backing_texture: renderTarget.m_webglPLSBackingR32UI.id(),
            level: 0,
            layer: 0,
            usage: GL_NONE,
        });
        if !capabilities.avoidTexture2DArrayWithWebGLPLS {
            recordGLCommand(GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                plane: CLIP_PLANE_IDX,
                backing_texture: renderTarget.m_webglPLSBackingR32UI.id(),
                level: 0,
                layer: 1,
                usage: GL_NONE,
            });
        } else {
            recordGLCommand(GLCommand::FramebufferTexturePixelLocalStorageANGLE {
                plane: CLIP_PLANE_IDX,
                backing_texture: renderTarget.m_webglPLSBackingR32UIFallback.id(),
                level: 0,
                layer: 0,
                usage: GL_NONE,
            });
        }
        recordGLCommand(GLCommand::FramebufferTexturePixelLocalStorageANGLE {
            plane: SCRATCH_COLOR_PLANE_IDX,
            backing_texture: renderTarget.m_webglPLSBackingRGBA8.id(),
            level: 0,
            layer: 0,
            usage: GL_NONE,
        });
        renderTarget.m_webglPLSBindingsDirty = false;
    }
}

pub(crate) fn bindTextureMSAAFramebuffer(
    renderTarget: &mut TextureRenderTargetGL,
    renderContextImpl: &mut RenderContextGLImpl,
    sampleCount: i32,
    preserveBounds: Option<&IAABB>,
    isFBO0: Option<&mut bool>,
) -> MSAAResolveAction {
    renderTarget.assertSameExecution(&renderContextImpl.rust_execution);
    let execution = renderTarget.executionStamp().clone();
    execution.withCurrent(|| {
        bindTextureMSAAFramebufferCurrent(
            renderTarget,
            renderContextImpl,
            sampleCount,
            preserveBounds,
            isFBO0,
        )
    })
}

fn bindTextureMSAAFramebufferCurrent(
    renderTarget: &mut TextureRenderTargetGL,
    renderContextImpl: &mut RenderContextGLImpl,
    mut sampleCount: i32,
    preserveBounds: Option<&IAABB>,
    isFBO0: Option<&mut bool>,
) -> MSAAResolveAction {
    debug_assert!(sampleCount > 0);
    if renderTarget.m_msaaFramebuffer.id() == 0 {
        renderTarget
            .m_msaaFramebuffer
            .moveAssign(Framebuffer::new());
    }

    if let Some(isFBO0) = isFBO0 {
        *isFBO0 = false;
    }

    sampleCount = sampleCount.max(1);
    if renderTarget.m_msaaFramebufferSampleCount != sampleCount {
        // Move assignment is source-significant: generate the replacement
        // name, then delete the previous renderbuffer before binding the new
        // one.
        renderTarget
            .m_msaaDepthStencilBuffer
            .moveAssign(Renderbuffer::new());
        recordGLCommand(GLCommand::BindRenderbuffer(
            GL_RENDERBUFFER,
            renderTarget.m_msaaDepthStencilBuffer.id(),
        ));

        recordGLCommand(GLCommand::BindFramebuffer(
            GL_FRAMEBUFFER,
            renderTarget.m_msaaFramebuffer.id(),
        ));

        // The native EXT_multisampled_render_to_texture attachment branch is
        // excluded by RIVE_WEBGL. WebGL always allocates the offscreen core
        // multisample renderbuffers here.
        recordGLCommand(GLCommand::RenderbufferStorageMultisample {
            target: GL_RENDERBUFFER,
            samples: sampleCount,
            internal_format: GL_DEPTH24_STENCIL8,
            width: renderTarget.width(),
            height: renderTarget.height(),
        });

        renderTarget
            .m_msaaColorBuffer
            .moveAssign(Renderbuffer::new());
        recordGLCommand(GLCommand::BindRenderbuffer(
            GL_RENDERBUFFER,
            renderTarget.m_msaaColorBuffer.id(),
        ));
        recordGLCommand(GLCommand::RenderbufferStorageMultisample {
            target: GL_RENDERBUFFER,
            samples: sampleCount,
            internal_format: GL_RGBA8,
            width: renderTarget.width(),
            height: renderTarget.height(),
        });
        recordGLCommand(GLCommand::FramebufferRenderbuffer {
            target: GL_FRAMEBUFFER,
            attachment: GL_COLOR_ATTACHMENT0,
            renderbuffer_target: GL_RENDERBUFFER,
            renderbuffer: renderTarget.m_msaaColorBuffer.id(),
        });
        recordGLCommand(GLCommand::FramebufferRenderbuffer {
            target: GL_FRAMEBUFFER,
            attachment: GL_DEPTH_STENCIL_ATTACHMENT,
            renderbuffer_target: GL_RENDERBUFFER,
            renderbuffer: renderTarget.m_msaaDepthStencilBuffer.id(),
        });

        renderTarget.m_msaaFramebufferSampleCount = sampleCount;
    }

    recordGLCommand(GLCommand::BindFramebuffer(
        GL_FRAMEBUFFER,
        renderTarget.m_msaaFramebuffer.id(),
    ));

    if renderContextImpl
        .capabilities()
        .EXT_multisampled_render_to_texture
    {
        MSAAResolveAction::automatic
    } else {
        if let Some(preserveBounds) = preserveBounds {
            renderContextImpl.blitTextureToFramebufferAsDraw(
                renderTarget.m_externalTextureID,
                preserveBounds,
                renderTarget.height(),
            );
        }
        MSAAResolveAction::framebufferBlit
    }
}

pub(crate) fn allocateTextureWebGLPLSBacking(
    renderTarget: &mut TextureRenderTargetGL,
    capabilities: &GLCapabilities,
) {
    let execution = renderTarget.executionStamp().clone();
    execution.withCurrent(|| allocateTextureWebGLPLSBackingCurrent(renderTarget, capabilities));
}

fn allocateTextureWebGLPLSBackingCurrent(
    renderTarget: &mut TextureRenderTargetGL,
    capabilities: &GLCapabilities,
) {
    if renderTarget.m_webglPLSBackingR32UI.id() == 0 {
        recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0));
        renderTarget
            .m_webglPLSBackingR32UI
            .moveAssign(Texture::new());
        if !capabilities.avoidTexture2DArrayWithWebGLPLS {
            recordGLCommand(GLCommand::BindTexture(
                GL_TEXTURE_2D_ARRAY,
                renderTarget.m_webglPLSBackingR32UI.id(),
            ));
            recordGLCommand(GLCommand::TexStorage3D {
                target: GL_TEXTURE_2D_ARRAY,
                levels: 1,
                internal_format: GL_R32UI,
                width: renderTarget.width(),
                height: renderTarget.height(),
                depth: 2,
            });
        } else {
            renderTarget
                .m_webglPLSBackingR32UIFallback
                .moveAssign(Texture::new());
            recordGLCommand(GLCommand::BindTexture(
                GL_TEXTURE_2D,
                renderTarget.m_webglPLSBackingR32UI.id(),
            ));
            recordGLCommand(GLCommand::TexStorage2D {
                target: GL_TEXTURE_2D,
                levels: 1,
                internal_format: GL_R32UI,
                width: renderTarget.width(),
                height: renderTarget.height(),
            });
            recordGLCommand(GLCommand::BindTexture(
                GL_TEXTURE_2D,
                renderTarget.m_webglPLSBackingR32UIFallback.id(),
            ));
            recordGLCommand(GLCommand::TexStorage2D {
                target: GL_TEXTURE_2D,
                levels: 1,
                internal_format: GL_R32UI,
                width: renderTarget.width(),
                height: renderTarget.height(),
            });
        }
        renderTarget.m_webglPLSBindingsDirty = true;
    }

    if renderTarget.m_webglPLSBackingRGBA8.id() == 0 {
        renderTarget
            .m_webglPLSBackingRGBA8
            .moveAssign(Texture::new());
        recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0));
        recordGLCommand(GLCommand::BindTexture(
            GL_TEXTURE_2D,
            renderTarget.m_webglPLSBackingRGBA8.id(),
        ));
        recordGLCommand(GLCommand::TexStorage2D {
            target: GL_TEXTURE_2D,
            levels: 1,
            internal_format: GL_RGBA8,
            width: renderTarget.width(),
            height: renderTarget.height(),
        });
        renderTarget.m_webglPLSBindingsDirty = true;
    }
}

pub(crate) fn bindFramebufferDestinationFramebuffer(
    renderTarget: &mut FramebufferRenderTargetGL,
    target: GLenum,
) {
    let execution = renderTarget.executionStamp().clone();
    execution.withCurrent(|| bindFramebufferDestinationFramebufferCurrent(renderTarget, target));
}

fn bindFramebufferDestinationFramebufferCurrent(
    renderTarget: &mut FramebufferRenderTargetGL,
    target: GLenum,
) {
    recordGLCommand(GLCommand::BindFramebuffer(
        target,
        renderTarget.m_externalFramebufferID,
    ));
}

pub(crate) fn allocateOffscreenTargetTexture(renderTarget: &mut FramebufferRenderTargetGL) {
    let execution = renderTarget.executionStamp().clone();
    execution.withCurrent(|| allocateOffscreenTargetTextureCurrent(renderTarget));
}

fn allocateOffscreenTargetTextureCurrent(renderTarget: &mut FramebufferRenderTargetGL) {
    if renderTarget.m_offscreenTargetTexture.id() == 0 {
        renderTarget
            .m_offscreenTargetTexture
            .moveAssign(Texture::new());
        recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0));
        recordGLCommand(GLCommand::BindTexture(
            GL_TEXTURE_2D,
            renderTarget.m_offscreenTargetTexture.id(),
        ));
        recordGLCommand(GLCommand::TexStorage2D {
            target: GL_TEXTURE_2D,
            levels: 1,
            internal_format: GL_RGBA8,
            width: renderTarget.width(),
            height: renderTarget.height(),
        });
        renderTarget
            .m_textureRenderTarget
            .setTargetTexture(renderTarget.m_offscreenTargetTexture.id());
    }
}

pub(crate) fn framebufferRenderTexture(renderTarget: &mut FramebufferRenderTargetGL) -> GLuint {
    let execution = renderTarget.executionStamp().clone();
    execution.withCurrent(|| {
        allocateOffscreenTargetTextureCurrent(renderTarget);
        renderTarget.m_textureRenderTarget.renderTexture()
    })
}

pub(crate) fn bindFramebufferTextureFramebuffer(
    renderTarget: &mut FramebufferRenderTargetGL,
    target: GLenum,
) {
    let execution = renderTarget.executionStamp().clone();
    execution.withCurrent(|| {
        allocateOffscreenTargetTextureCurrent(renderTarget);
        renderTarget
            .m_textureRenderTarget
            .bindTextureFramebuffer(target);
    });
}

pub(crate) fn bindFramebufferHeadlessFramebuffer(
    renderTarget: &mut FramebufferRenderTargetGL,
    capabilities: &GLCapabilities,
) {
    let execution = renderTarget.executionStamp().clone();
    execution.withCurrent(|| {
        renderTarget
            .m_textureRenderTarget
            .bindHeadlessFramebuffer(capabilities);
    });
}

pub(crate) fn bindFramebufferMSAAFramebuffer(
    renderTarget: &mut FramebufferRenderTargetGL,
    renderContextImpl: &mut RenderContextGLImpl,
    sampleCount: i32,
    preserveBounds: Option<&IAABB>,
    isFBO0: Option<&mut bool>,
) -> MSAAResolveAction {
    renderTarget.assertSameExecution(&renderContextImpl.rust_execution);
    let execution = renderTarget.executionStamp().clone();
    execution.withCurrent(|| {
        bindFramebufferMSAAFramebufferCurrent(
            renderTarget,
            renderContextImpl,
            sampleCount,
            preserveBounds,
            isFBO0,
        )
    })
}

fn bindFramebufferMSAAFramebufferCurrent(
    renderTarget: &mut FramebufferRenderTargetGL,
    renderContextImpl: &mut RenderContextGLImpl,
    sampleCount: i32,
    preserveBounds: Option<&IAABB>,
    isFBO0: Option<&mut bool>,
) -> MSAAResolveAction {
    debug_assert!(sampleCount > 0);
    if renderTarget.m_sampleCount > 1 {
        // The external framebuffer's actual sample count is authoritative even
        // when it differs from the requested count.
        bindFramebufferDestinationFramebufferCurrent(renderTarget, GL_FRAMEBUFFER);
        if let Some(isFBO0) = isFBO0 {
            *isFBO0 = renderTarget.m_externalFramebufferID == 0;
        }
        MSAAResolveAction::automatic
    } else {
        if let Some(preserveBounds) = preserveBounds {
            allocateOffscreenTargetTextureCurrent(renderTarget);
            renderTarget
                .m_textureRenderTarget
                .bindTextureFramebuffer(GL_DRAW_FRAMEBUFFER);
            bindFramebufferDestinationFramebufferCurrent(renderTarget, GL_READ_FRAMEBUFFER);
            renderContextImpl
                .state()
                .borrow_mut()
                .setPipelineState(&COLOR_ONLY_PIPELINE_STATE, ScissorAction::disable);
            super::gl_utils_impl::BlitFramebuffer(
                *preserveBounds,
                renderTarget.height(),
                GL_COLOR_BUFFER_BIT,
            );
        } else if renderContextImpl
            .capabilities()
            .EXT_multisampled_render_to_texture
        {
            allocateOffscreenTargetTextureCurrent(renderTarget);
        }

        // The nested action is deliberately ignored: rendering is offscreen
        // from the outer client's perspective, so the client always blits.
        let _ = renderTarget.m_textureRenderTarget.bindMSAAFramebuffer(
            renderContextImpl,
            sampleCount,
            preserveBounds,
            isFBO0,
        );
        MSAAResolveAction::framebufferBlit
    }
}

pub(crate) fn allocateFramebufferWebGLPLSBacking(
    renderTarget: &mut FramebufferRenderTargetGL,
    capabilities: &GLCapabilities,
) {
    let execution = renderTarget.executionStamp().clone();
    execution.withCurrent(|| {
        renderTarget
            .m_textureRenderTarget
            .allocateWebGLPLSBacking(capabilities);
    });
}

unsafe fn dropRenderTargetGLSourceFields(renderTarget: &mut RenderTargetGL, deleteNames: bool) {
    if !deleteNames {
        renderTarget.m_dstColorFramebuffer.0.m_id = 0;
    }
    unsafe { ManuallyDrop::drop(&mut renderTarget.m_dstColorFramebuffer) };
    if !deleteNames {
        renderTarget.m_dstColorTexture.0.m_id = 0;
    }
    unsafe {
        ManuallyDrop::drop(&mut renderTarget.m_dstColorTexture);
        ManuallyDrop::drop(&mut renderTarget.lite_rtti);
        ManuallyDrop::drop(&mut renderTarget.base);
    }
}

unsafe fn dropTextureRenderTargetGLSourceFields(
    renderTarget: &mut TextureRenderTargetGL,
    deleteNames: bool,
) {
    if !deleteNames {
        renderTarget.m_msaaDepthStencilBuffer.0.m_id = 0;
    }
    unsafe { ManuallyDrop::drop(&mut renderTarget.m_msaaDepthStencilBuffer) };
    if !deleteNames {
        renderTarget.m_msaaColorBuffer.0.m_id = 0;
    }
    unsafe { ManuallyDrop::drop(&mut renderTarget.m_msaaColorBuffer) };
    if !deleteNames {
        renderTarget.m_msaaFramebuffer.0.m_id = 0;
    }
    unsafe { ManuallyDrop::drop(&mut renderTarget.m_msaaFramebuffer) };
    if !deleteNames {
        renderTarget.m_webglPLSBackingRGBA8.0.m_id = 0;
    }
    unsafe { ManuallyDrop::drop(&mut renderTarget.m_webglPLSBackingRGBA8) };
    if !deleteNames {
        renderTarget.m_webglPLSBackingR32UIFallback.0.m_id = 0;
    }
    unsafe { ManuallyDrop::drop(&mut renderTarget.m_webglPLSBackingR32UIFallback) };
    if !deleteNames {
        renderTarget.m_webglPLSBackingR32UI.0.m_id = 0;
    }
    unsafe { ManuallyDrop::drop(&mut renderTarget.m_webglPLSBackingR32UI) };
    if !deleteNames {
        renderTarget.m_headlessFramebuffer.0.m_id = 0;
    }
    unsafe { ManuallyDrop::drop(&mut renderTarget.m_headlessFramebuffer) };
    if !deleteNames {
        renderTarget.m_framebufferID.0.m_id = 0;
    }
    unsafe {
        ManuallyDrop::drop(&mut renderTarget.m_framebufferID);
        ManuallyDrop::drop(&mut renderTarget.base);
    }
}

unsafe fn dropFramebufferRenderTargetGLSourceFields(
    renderTarget: &mut FramebufferRenderTargetGL,
    deleteNames: bool,
) {
    if !deleteNames {
        renderTarget.m_offscreenTargetTexture.0.m_id = 0;
    }
    unsafe {
        ManuallyDrop::drop(&mut renderTarget.m_offscreenTargetTexture);
        ManuallyDrop::drop(&mut renderTarget.m_textureRenderTarget);
        ManuallyDrop::drop(&mut renderTarget.base);
    }
}

impl Drop for RenderTargetGL {
    fn drop(&mut self) {
        let execution = self.executionStamp().clone();
        let deleted =
            execution.withDeleteCurrent(|| unsafe { dropRenderTargetGLSourceFields(self, true) });
        if deleted.is_none() {
            // Context loss advances the generation. Numeric names from the old
            // context are quarantined, while all Rust fields still dismantle.
            unsafe { dropRenderTargetGLSourceFields(self, false) };
        }
        unsafe {
            ManuallyDrop::drop(&mut self.rust_execution);
        }
    }
}

impl Drop for TextureRenderTargetGL {
    fn drop(&mut self) {
        let execution = self.executionStamp().clone();
        let deleted = execution
            .withDeleteCurrent(|| unsafe { dropTextureRenderTargetGLSourceFields(self, true) });
        if deleted.is_none() {
            unsafe { dropTextureRenderTargetGLSourceFields(self, false) };
        }
    }
}

impl Drop for FramebufferRenderTargetGL {
    fn drop(&mut self) {
        let execution = self.executionStamp().clone();
        let deleted = execution
            .withDeleteCurrent(|| unsafe { dropFramebufferRenderTargetGLSourceFields(self, true) });
        if deleted.is_none() {
            unsafe { dropFramebufferRenderTargetGLSourceFields(self, false) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::include::rive::refcnt_hpp::RefCntTarget;
    use std::cell::RefCell;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::rc::Rc;

    #[derive(Default)]
    struct ProviderLog {
        commands: Vec<GLCommand>,
        generated: Vec<(GLObjectKind, GLuint)>,
    }

    struct TestProvider {
        log: Rc<RefCell<ProviderLog>>,
        nextName: GLuint,
        lifecycleIngress: Option<GLContextLifecycleIngress>,
        finalReleaseIngress: Option<GLFinalReleaseIngress>,
    }

    impl TestProvider {
        fn allocate(&mut self) -> GLuint {
            let name = self.nextName;
            self.nextName += 13;
            name
        }
    }

    impl GLExecutionProvider for TestProvider {
        fn installContextLifecycleIngress(&mut self, ingress: GLContextLifecycleIngress) {
            assert!(self.lifecycleIngress.replace(ingress).is_none());
        }

        fn installFinalReleaseIngress(
            &mut self,
            ingress: GLFinalReleaseIngress,
        ) -> std::sync::Arc<dyn nuxie_ore_metal::gpu_resource::ResourceFinalReleaseWake> {
            assert!(self.finalReleaseIngress.replace(ingress).is_none());
            std::sync::Arc::new(TestFinalReleaseWake::default())
        }

        fn submit(&mut self, command: GLCommand) {
            self.log.borrow_mut().commands.push(command);
        }

        fn generateObject(&mut self, kind: GLObjectKind) -> GLuint {
            let name = self.allocate();
            self.log.borrow_mut().generated.push((kind, name));
            name
        }

        fn createProgram(&mut self) -> GLuint {
            self.allocate()
        }

        fn createShader(&mut self, _shaderType: GLenum) -> GLuint {
            self.allocate()
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

    fn domain(startName: GLuint) -> (GLExecutionDomain, Rc<RefCell<ProviderLog>>) {
        let log = Rc::new(RefCell::new(ProviderLog::default()));
        let domain = GLExecutionDomain::new(Box::new(TestProvider {
            log: log.clone(),
            nextName: startName,
            lifecycleIngress: None,
            finalReleaseIngress: None,
        }));
        (domain, log)
    }

    #[test]
    fn no_ambient_scope_uses_the_targets_creation_domain() {
        let (domain, log) = domain(101);
        let mut target = TextureRenderTargetGL::new(8, 6, domain.stamp());
        target.setTargetTexture(701);

        // There is no ambient GL domain here. The target must install its own
        // stamp for the complete lazy-allocation/attachment operation.
        target.bindTextureFramebuffer(GL_FRAMEBUFFER);

        assert_eq!(log.borrow().generated, [(GLObjectKind::Framebuffer, 101)]);
        assert_eq!(
            log.borrow().commands,
            [
                GLCommand::BindFramebuffer(GL_FRAMEBUFFER, 101),
                GLCommand::FramebufferTexture2D {
                    target: GL_FRAMEBUFFER,
                    attachment: GL_COLOR_ATTACHMENT0,
                    texture_target: GL_TEXTURE_2D,
                    texture: 701,
                    level: 0,
                },
            ]
        );
    }

    #[test]
    fn foreign_ambient_domain_is_isolated_and_identity_mismatch_is_rejected() {
        let (ownerDomain, ownerLog) = domain(201);
        let (foreignDomain, foreignLog) = domain(901);
        let mut target = TextureRenderTargetGL::new(4, 4, ownerDomain.stamp());
        target.setTargetTexture(702);

        foreignDomain.withCurrent(|| {
            target.bindDestinationFramebuffer(GL_FRAMEBUFFER);
            recordGLCommand(GLCommand::Clear(GL_COLOR_BUFFER_BIT));
        });

        assert!(matches!(
            ownerLog.borrow().commands.as_slice(),
            [
                GLCommand::BindFramebuffer(GL_FRAMEBUFFER, 201),
                GLCommand::FramebufferTexture2D { texture: 702, .. }
            ]
        ));
        assert_eq!(
            foreignLog.borrow().commands,
            [GLCommand::Clear(GL_COLOR_BUFFER_BIT)]
        );
        let foreignStamp = foreignDomain.stamp();
        assert!(catch_unwind(AssertUnwindSafe(
            || target.assertSameExecution(&foreignStamp)
        ))
        .is_err());
    }

    #[test]
    fn stale_generation_tears_down_fields_without_deleting_recycled_names() {
        let (domain, log) = domain(301);
        let mut target = TextureRenderTargetGL::new(3, 3, domain.stamp());
        target.m_framebufferID.0.m_id = 31;
        target.m_msaaColorBuffer.0.m_id = 32;
        target.base.m_dstColorTexture.0.m_id = 33;

        domain.markContextLost();
        log.borrow_mut().commands.clear();
        drop(target);

        assert!(log.borrow().commands.is_empty());
        domain.shutdown();
    }

    #[test]
    fn texture_target_deletes_owned_names_in_exact_reverse_source_order() {
        let (domain, log) = domain(401);
        let mut target = TextureRenderTargetGL::new(3, 3, domain.stamp());
        target.m_externalTextureID = 999;
        target.m_framebufferID.0.m_id = 11;
        target.m_headlessFramebuffer.0.m_id = 12;
        target.m_webglPLSBackingR32UI.0.m_id = 13;
        target.m_webglPLSBackingR32UIFallback.0.m_id = 14;
        target.m_webglPLSBackingRGBA8.0.m_id = 15;
        target.m_msaaFramebuffer.0.m_id = 16;
        target.m_msaaColorBuffer.0.m_id = 17;
        target.m_msaaDepthStencilBuffer.0.m_id = 18;
        target.base.m_dstColorTexture.0.m_id = 19;
        target.base.m_dstColorFramebuffer.0.m_id = 20;

        drop(target);

        assert_eq!(
            log.borrow().commands,
            [
                GLCommand::DeleteRenderbuffer(18),
                GLCommand::DeleteRenderbuffer(17),
                GLCommand::DeleteFramebuffer(16),
                GLCommand::DeleteTexture(15),
                GLCommand::DeleteTexture(14),
                GLCommand::DeleteTexture(13),
                GLCommand::DeleteFramebuffer(12),
                GLCommand::DeleteFramebuffer(11),
                GLCommand::DeleteFramebuffer(20),
                GLCommand::DeleteTexture(19),
            ]
        );
        assert!(!log.borrow().commands.iter().any(|command| matches!(
            command,
            GLCommand::DeleteTexture(999) | GLCommand::DeleteFramebuffer(999)
        )));
    }

    #[test]
    fn client_texture_and_framebuffer_ids_remain_borrowed() {
        let (domain, log) = domain(501);
        let mut textureTarget = TextureRenderTargetGL::new(2, 2, domain.stamp());
        textureTarget.setTargetTexture(801);
        let framebufferTarget = FramebufferRenderTargetGL::new(2, 2, 802, 4, domain.stamp());

        drop(textureTarget);
        drop(framebufferTarget);

        assert!(log.borrow().commands.is_empty());
    }

    #[test]
    fn framebuffer_target_drops_outer_texture_before_embedded_target_and_base() {
        let (domain, log) = domain(551);
        let mut target = FramebufferRenderTargetGL::new(2, 2, 900, 1, domain.stamp());
        target.m_offscreenTargetTexture.0.m_id = 41;
        target.m_textureRenderTarget.m_framebufferID.0.m_id = 42;
        target.m_textureRenderTarget.base.m_dstColorTexture.0.m_id = 43;
        target.base.m_dstColorFramebuffer.0.m_id = 44;

        drop(target);

        assert_eq!(
            log.borrow().commands,
            [
                GLCommand::DeleteTexture(41),
                GLCommand::DeleteFramebuffer(42),
                GLCommand::DeleteTexture(43),
                GLCommand::DeleteFramebuffer(44),
            ]
        );
        assert!(!log
            .borrow()
            .commands
            .contains(&GLCommand::DeleteFramebuffer(900)));
    }

    #[test]
    fn render_target_zero_release_routes_complete_drop_to_owner_scope() {
        let (domain, log) = domain(601);
        let mut target = Box::new(TextureRenderTargetGL::new(2, 2, domain.stamp()));
        target.m_framebufferID.0.m_id = 77;
        let targetAddress = Box::into_raw(target).cast::<RenderTarget>() as usize;
        domain.retireRenderer();
        assert!(domain.isRendererRetired());
        assert!(
            domain.isLive(),
            "normal renderer retirement retains the target's GL generation"
        );

        std::thread::spawn(move || unsafe {
            (&*(targetAddress as *const RenderTarget)).unref();
        })
        .join()
        .expect("worker release completes");

        // The worker may enqueue only; it cannot execute the Rc/GL destructor.
        assert!(log.borrow().commands.is_empty());
        domain.withCurrent(|| {});
        assert_eq!(log.borrow().commands, [GLCommand::DeleteFramebuffer(77)]);
    }
}
