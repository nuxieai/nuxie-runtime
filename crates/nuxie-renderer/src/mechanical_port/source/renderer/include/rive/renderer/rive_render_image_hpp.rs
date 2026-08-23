/*
 * Copyright 2023 Rive
 */

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/rive_render_image.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// /*
//  * Copyright 2023 Rive
//  */
//
// #pragma once
//
// #include "rive/renderer.hpp"
// #include "rive/renderer/texture.hpp"
// #ifdef __EMSCRIPTEN__
// #include <emscripten.h>
// #endif
//
// namespace rive
// {
// class RiveRenderImage : public LITE_RTTI_OVERRIDE(RenderImage, RiveRenderImage)
// {
// public:
//     RiveRenderImage(rcp<gpu::Texture> texture) :
//         RiveRenderImage(texture->width(), texture->height())
//     {
//         resetTexture(std::move(texture));
//     }
//
//     rcp<gpu::Texture> refTexture() const { return m_texture; }
//     gpu::Texture* getTexture() { return m_texture.get(); }
//
// protected:
//     RiveRenderImage(int width, int height)
//     {
//         m_Width = width;
//         m_Height = height;
//     }
//
//     void resetTexture(rcp<gpu::Texture> texture = nullptr)
//     {
//         assert(texture == nullptr || texture->width() == m_Width);
//         assert(texture == nullptr || texture->height() == m_Height);
//         m_texture = std::move(texture);
//     }
//
//     // Used by the android runtime to send m_texture off to the worker thread to
//     // be deleted.
//     gpu::Texture* releaseTexture() { return m_texture.release(); }
//
// private:
//     rcp<gpu::Texture> m_texture;
// };
// } // namespace rive

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use crate::mechanical_port::source::include::rive::refcnt_hpp::{
    rcp, static_rcp_cast, RefCntTarget,
};
use crate::mechanical_port::source::include::rive::renderer_hpp::RenderImage;
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{
    LiteRttiBase, LiteRttiCastFrom, LiteRttiTypeId,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_buffer_hpp::RenderResourceDomain;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;
use core::mem::ManuallyDrop;
use nuxie_render_api::Mat2D;
use std::any::Any;
use std::rc::Rc;

// `RiveRenderImage` is declared in namespace `rive`; its texture dependency
// remains the source-shaped `rive::gpu::Texture` owner from texture.hpp.  That
// owner carries immutable dimensions and the process-global resource hash;
// this header deliberately retains one intrusive `rcp<Texture>` rather than
// copying either value into the image.
//
// The C++ base is `LITE_RTTI_OVERRIDE(RenderImage, RiveRenderImage)`, whose
// `RenderImage` base contains `RefCnt<RenderImage>`, the lite-RTTI identity,
// and the protected image metadata. Rust has no base subobjects, so those
// source fields are flattened in declaration order below. The concrete
// `RefCnt<RiveRenderImage>` target preserves the source's virtual destruction
// boundary while retaining the same one-reference owner semantics.
#[repr(C)]
pub struct RiveRenderImage {
    // public LITE_RTTI_OVERRIDE(RenderImage, RiveRenderImage) base:
    // RefCnt<RenderImage> is represented by a concrete target so that the
    // source zero-reference delete reaches this complete derived owner.
    pub(crate) base: RenderImage,

    // private:
    // rcp<gpu::Texture> m_texture;
    // Explicit derived Drop releases this owner before the offset-zero base.
    m_texture: ManuallyDrop<rcp<Texture>>,
}

// SAFETY: the embedded RenderImage base begins at offset zero and its installed
// complete destructor recovers the enclosing RiveRenderImage allocation.
unsafe impl RefCntTarget for RiveRenderImage {
    // RefCnt's inherited ref() operation.
    fn r#ref(&self) {
        RefCntTarget::r#ref(&self.base);
    }

    // RefCnt's inherited unref() operation.
    unsafe fn unref(&self) {
        unsafe { RefCntTarget::unref(&self.base) };
    }
}

impl LiteRttiBase for RiveRenderImage {
    fn liteTypeID(&self) -> u32 {
        self.base.liteTypeID()
    }
    fn setLiteTypeID(&mut self, id: u32) {
        self.base.setLiteTypeID(id);
    }
}

impl LiteRttiTypeId for RiveRenderImage {
    const LITE_RTTI_TYPE_ID: u32 = RiveRenderImage::LITE_RTTI_TYPE_ID;
}

impl LiteRttiCastFrom<RenderImage> for RiveRenderImage {
    unsafe fn from_base(base: *mut RenderImage) -> *mut Self {
        // repr(C), offset-zero RenderImage base, and the matching stored type
        // ID establish the same checked static_cast used by lite_rtti_cast.
        base.cast()
    }
}

impl RiveRenderImage {
    // constexpr static uint32_t LITE_RTTI_TYPE_ID =
    //     CONST_ID(RiveRenderImage);
    pub const LITE_RTTI_TYPE_ID: u32 =
        crate::mechanical_port::source::include::utils::lite_rtti_hpp::CONST_ID("RiveRenderImage");

    // RiveRenderImage(rcp<gpu::Texture> texture) :
    //     RiveRenderImage(texture->width(), texture->height())
    // {
    //     resetTexture(std::move(texture));
    // }
    /// Constructs an image from a live texture owner.
    ///
    /// # Safety
    /// `texture` must be non-null and point to a live complete `Texture`
    /// allocation. The source constructor reads its dimensions before moving
    /// the owner and has no null branch.
    pub unsafe fn new(mut texture: rcp<Texture>) -> Self {
        // The source dereferences texture before moving it and has no null
        // branch. Keep that quirk and evaluation order at this mechanical
        // boundary; a null rcp is therefore an invalid source call.
        let width = unsafe { (&*texture.get()).width() as i32 };
        let height = unsafe { (&*texture.get()).height() as i32 };
        let mut image = Self::new_with_dimensions(width, height);
        image.resetTexture(texture);
        image
    }

    // rcp<gpu::Texture> refTexture() const { return m_texture; }
    // Returning a cloned rcp performs the source copy-constructor retain.
    pub fn refTexture(&self) -> rcp<Texture> {
        rcp::copy_ctor(&self.m_texture)
    }

    // gpu::Texture* getTexture() { return m_texture.get(); }
    pub fn getTexture(&mut self) -> *mut Texture {
        self.m_texture.get()
    }

    // Inherited RenderImage metadata accessors remain part of the public
    // surface consumed through a RiveRenderImage pointer.
    // int width() const { return m_Width; }
    pub fn width(&self) -> i32 {
        self.base.m__width
    }

    // int height() const { return m_Height; }
    pub fn height(&self) -> i32 {
        self.base.m__height
    }

    // const Mat2D& uvTransform() const { return m_uvTransform; }
    pub fn uvTransform(&self) -> &Mat2D {
        &self.base.m_uv_transform
    }

    // The inherited lite-RTTI query is observable through the source base
    // contract and returns the most-derived RiveRenderImage identity.
    pub fn liteTypeID(&self) -> u32 {
        self.base.m_liteTypeId
    }

    // protected:
    // RiveRenderImage(int width, int height)
    // {
    //     m_Width = width;
    //     m_Height = height;
    // }
    pub(crate) fn new_with_dimensions(width: i32, height: i32) -> Self {
        unsafe fn destroy_complete(base: *mut RenderImage) {
            unsafe { drop(Box::from_raw(base.cast::<RiveRenderImage>())) };
        }
        let mut base = RenderImage::new();
        base.destroy_complete = destroy_complete;
        base.m_liteTypeId = Self::LITE_RTTI_TYPE_ID;
        base.m__width = width;
        base.m__height = height;
        Self {
            base,
            m_texture: ManuallyDrop::new(rcp::new()),
        }
    }

    // void resetTexture(rcp<gpu::Texture> texture = nullptr)
    // {
    //     assert(texture == nullptr || texture->width() == m_Width);
    //     assert(texture == nullptr || texture->height() == m_Height);
    //     m_texture = std::move(texture);
    // }
    pub(crate) fn resetTexture(&mut self, texture: rcp<Texture>) {
        if !texture.get().is_null() {
            // C++ performs the comparison after the uint32_t/int usual
            // arithmetic conversion. The cast preserves that source-width
            // comparison, including the signed-dimension quirk.
            debug_assert!(unsafe { (&*texture.get()).width() } == self.base.m__width as u32);
            debug_assert!(unsafe { (&*texture.get()).height() } == self.base.m__height as u32);
        }
        unsafe { ManuallyDrop::drop(&mut self.m_texture) };
        self.m_texture = ManuallyDrop::new(texture);
    }

    // Rust has no default arguments. This explicit null call is the source
    // `texture = nullptr` path and retains the same nullable rcp state.
    pub(crate) fn resetTextureDefault(&mut self) {
        self.resetTexture(rcp::new());
    }

    // // Used by the android runtime to send m_texture off to the worker thread to
    // // be deleted.
    // gpu::Texture* releaseTexture() { return m_texture.release(); }
    pub(crate) fn releaseTexture(&mut self) -> *mut Texture {
        self.m_texture.release()
    }
}

impl Drop for RiveRenderImage {
    fn drop(&mut self) {
        // Destroy the derived member before the flattened inherited base.
        unsafe { ManuallyDrop::drop(&mut self.m_texture) };
    }
}

impl nuxie_render_api::RenderImage for RiveRenderImage {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
    fn width(&self) -> u32 {
        self.width().max(0) as u32
    }
    fn height(&self) -> u32 {
        self.height().max(0) as u32
    }
    fn uv_transform(&self) -> Mat2D {
        *self.uvTransform()
    }
}

/// Immutable product handle for the exact intrusive RiveRenderImage produced
/// by RenderContext image decoding. Cloning this wrapper performs the source
/// rcp retain; it never copies or boxes the complete image allocation.
#[derive(Clone)]
struct AttachedImageExecutionDomain {
    resource_domain: RenderResourceDomain,
    // Retains the actual backend execution owner until after source teardown.
    _domain_guard: Rc<dyn Any>,
}

pub struct RiveRenderImageHandle {
    source: rcp<RiveRenderImage>,
    // Declared after source so image/texture release completes before the
    // backend owner drops. Identity and lifetime are one indivisible edge.
    execution_domain: Option<AttachedImageExecutionDomain>,
}

impl RiveRenderImageHandle {
    /// Adopts one exact intrusive RiveRenderImage retain. The image product
    /// surface is immutable, so additional source rcp owners remain safe.
    pub fn from_exact(source: rcp<RiveRenderImage>) -> Option<Self> {
        (!source.get().is_null()).then_some(Self {
            source,
            execution_domain: None,
        })
    }

    /// # Safety
    /// The nonnull source owner must be the offset-zero RiveRenderImage
    /// allocation returned by RenderContext::decodeImage.
    pub(crate) unsafe fn from_source(source: rcp<RenderImage>) -> Option<Self> {
        if source.get().is_null() {
            return None;
        }
        let source = unsafe { static_rcp_cast(source) };
        Self::from_exact(source)
    }

    /// Attach the opaque identity and owner of this image's execution domain
    /// together. The consuming builder permits this attachment exactly once.
    pub(crate) fn with_execution_domain(
        mut self,
        resource_domain: RenderResourceDomain,
        domain_guard: Rc<dyn Any>,
    ) -> Self {
        assert!(
            self.execution_domain.is_none(),
            "image execution domain already attached"
        );
        self.execution_domain = Some(AttachedImageExecutionDomain {
            resource_domain,
            _domain_guard: domain_guard,
        });
        self
    }

    /// Returns whether this resource was created by the queried execution
    /// domain. An unattached source handle belongs to no product domain.
    pub(crate) fn belongs_to(&self, resource_domain: &RenderResourceDomain) -> bool {
        self.execution_domain
            .as_ref()
            .is_some_and(|attached| attached.resource_domain.same_domain(resource_domain))
    }

    fn source(&self) -> &RiveRenderImage {
        // SAFETY: from_source rejects null and this handle owns a retain.
        unsafe { &*self.source.get() }
    }

    /// Borrows the exact source base only after validating the execution
    /// domain. The borrow cannot outlive this handle and therefore cannot
    /// outlive the bundled lifetime guard.
    pub(crate) fn source_base_for(
        &self,
        resource_domain: &RenderResourceDomain,
    ) -> Option<&RenderImage> {
        self.belongs_to(resource_domain)
            .then(|| &self.source().base)
    }
}

impl Clone for RiveRenderImageHandle {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            execution_domain: self.execution_domain.clone(),
        }
    }
}

impl nuxie_render_api::RenderImage for RiveRenderImageHandle {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
    fn width(&self) -> u32 {
        self.source().width().max(0) as u32
    }
    fn height(&self) -> u32 {
        self.source().height().max(0) as u32
    }
    fn uv_transform(&self) -> Mat2D {
        *self.source().uvTransform()
    }
}
