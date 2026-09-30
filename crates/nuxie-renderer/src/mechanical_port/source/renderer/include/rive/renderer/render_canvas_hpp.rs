//! Exact owner translation of renderer/include/rive/renderer/render_canvas.hpp.
//! Upstream 57dddb3727306e284773ec20c653cf686c45abee.
#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals
)]

use crate::deferred::cmd::deferred_canvas::CanvasRetirer;
use crate::mechanical_port::source::include::rive::refcnt_hpp::{
    make_rcp, rcp, RefCnt, RefCntTarget,
};
use crate::mechanical_port::source::include::rive::renderer_hpp::RenderImage;
use crate::mechanical_port::source::renderer::include::rive::renderer::{
    render_target_hpp::RenderTarget, rive_render_image_hpp::RiveRenderImage, texture_hpp::Texture,
};
use core::mem::ManuallyDrop;
use std::sync::Arc;

/// Stable image identity exists before any replay device supplies its pixels.
#[repr(C)]
pub struct RenderCanvasImage {
    base: RiveRenderImage,
    // Flattened DeferredRenderCanvasImage derived owner. Its base remains first
    // so retained RenderImage handles invoke the complete final destructor.
    retirer: Option<Arc<CanvasRetirer>>,
}
impl RenderCanvasImage {
    pub fn new(width: u32, height: u32) -> Self {
        unsafe fn destroy_complete(base: *mut RenderImage) {
            unsafe { drop(Box::from_raw(base.cast::<RenderCanvasImage>())) };
        }
        let mut base = RiveRenderImage::new_with_dimensions(width as i32, height as i32);
        base.base.destroy_complete = destroy_complete;
        Self {
            base,
            retirer: None,
        }
    }
    pub(crate) fn new_deferred(retirer: Arc<CanvasRetirer>, width: u32, height: u32) -> Self {
        let mut image = Self::new(width, height);
        image.retirer = Some(retirer);
        image
    }
    pub fn resetTexture(&mut self, texture: rcp<Texture>) {
        self.base.resetTexture(texture);
    }
}
impl Drop for RenderCanvasImage {
    fn drop(&mut self) {
        if let Some(retirer) = &self.retirer {
            let texture = self.base.refTexture();
            retirer.retire(
                (!texture.get().is_null()).then(|| Box::new(texture) as Box<dyn std::any::Any>),
                None,
            );
            self.base.resetTexture(rcp::new());
        }
    }
}
// SAFETY: the offset-zero image base retains this complete owner's destructor.
unsafe impl RefCntTarget for RenderCanvasImage {
    fn r#ref(&self) {
        self.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
}

/// Construction allocates no GPU resources. The replay device supplies the
/// texture and render target, without changing this canvas or image identity.
#[repr(C)]
pub struct RenderCanvas {
    pub(crate) base: RefCnt<RenderCanvas>,
    m_renderImage: ManuallyDrop<rcp<RenderCanvasImage>>,
    m_renderTarget: ManuallyDrop<rcp<RenderTarget>>,
    m_width: u32,
    m_height: u32,
    // Flattened DeferredRenderCanvas derived-owner state.
    retirer: Option<Arc<CanvasRetirer>>,
}
// SAFETY: the intrusive base is the first field of this complete owner.
unsafe impl RefCntTarget for RenderCanvas {
    fn r#ref(&self) {
        self.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
}
impl RenderCanvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self::with_image(
            make_rcp(|| RenderCanvasImage::new(width, height)),
            width,
            height,
        )
    }
    fn with_image(image: rcp<RenderCanvasImage>, width: u32, height: u32) -> Self {
        Self {
            base: RefCnt::new(),
            m_renderImage: ManuallyDrop::new(image),
            m_renderTarget: ManuallyDrop::new(rcp::new()),
            m_width: width,
            m_height: height,
            retirer: None,
        }
    }
    pub(crate) fn new_deferred(retirer: Arc<CanvasRetirer>, width: u32, height: u32) -> Self {
        let image = make_rcp(|| RenderCanvasImage::new_deferred(retirer.clone(), width, height));
        let mut canvas = Self::with_image(image, width, height);
        canvas.retirer = Some(retirer);
        canvas
    }
    fn release_render_target(&mut self) -> rcp<RenderTarget> {
        std::mem::replace(&mut *self.m_renderTarget, rcp::new())
    }
    pub fn width(&self) -> u32 {
        self.m_width
    }
    pub fn height(&self) -> u32 {
        self.m_height
    }
    pub fn isBacked(&self) -> bool {
        !self.m_renderTarget.get().is_null()
    }
    pub fn setBacking(&mut self, texture: rcp<Texture>, renderTarget: rcp<RenderTarget>) {
        assert!(!self.isBacked());
        // SAFETY: the canvas retains a non-null image for its complete lifetime.
        unsafe { (&mut *self.m_renderImage.get()).resetTexture(texture) };
        unsafe { ManuallyDrop::drop(&mut self.m_renderTarget) };
        self.m_renderTarget = ManuallyDrop::new(renderTarget);
    }
    pub fn renderImage(&mut self) -> *mut RiveRenderImage {
        self.m_renderImage.get().cast::<RiveRenderImage>()
    }
    pub(crate) fn render_image_ref(&self) -> &RiveRenderImage {
        unsafe { &*self.m_renderImage.get().cast::<RiveRenderImage>() }
    }
    pub(crate) fn ref_render_image(&self) -> rcp<RiveRenderImage> {
        // SAFETY: RenderCanvasImage's offset-zero base is RiveRenderImage.
        unsafe { rcp::converting_copy_ctor(&self.m_renderImage) }
    }
    pub fn renderTarget(&mut self) -> *mut RenderTarget {
        self.m_renderTarget.get()
    }
    pub(crate) fn render_target_ref(&self) -> &RenderTarget {
        assert!(self.isBacked(), "canvas must be backed before rendering");
        unsafe { &*self.m_renderTarget.get() }
    }
}
impl Drop for RenderCanvas {
    fn drop(&mut self) {
        if let Some(retirer) = self.retirer.clone() {
            let target = self.release_render_target();
            retirer.retire(
                None,
                (!target.get().is_null()).then(|| Box::new(target) as Box<dyn std::any::Any>),
            );
        }
        // Source reverse member destruction precedes intrusive base destruction.
        unsafe {
            ManuallyDrop::drop(&mut self.m_renderTarget);
            ManuallyDrop::drop(&mut self.m_renderImage);
        }
    }
}
