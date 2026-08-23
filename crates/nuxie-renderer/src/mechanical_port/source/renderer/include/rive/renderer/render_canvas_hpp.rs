/*
 * Copyright 2025 Rive
 */

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/render_canvas.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// #pragma once

// #include "rive/refcnt.hpp"
// #include "rive/renderer/render_target.hpp"
// #include "rive/renderer/rive_render_image.hpp"

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use crate::mechanical_port::source::include::rive::refcnt_hpp::{rcp, RefCnt, RefCntTarget};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImage;
use core::mem::ManuallyDrop;

// namespace rive::gpu
// {
//
// A GPU texture that can be used as both a render target (for rendering into)
// and a render image (for compositing into Rive draws). Enables off-screen
// rendering for 3D content (Ore), cached 2D content, or any render-to-texture
// use case.
// class RenderCanvas : public RefCnt<RenderCanvas>
// {
// public:
//     RenderCanvas(rcp<RiveRenderImage> image, rcp<RenderTarget> target) :
//         m_renderImage(std::move(image)), m_renderTarget(std::move(target))
//     {}
//
//     uint32_t width() const { return m_renderTarget->width(); }
//     uint32_t height() const { return m_renderTarget->height(); }
//
//     // Use as a RenderImage for compositing into Rive draws.
//     RiveRenderImage* renderImage() { return m_renderImage.get(); }
//
//     // Use as a RenderTarget for rendering into this texture.
//     RenderTarget* renderTarget() { return m_renderTarget.get(); }
//
// private:
//     rcp<RiveRenderImage> m_renderImage;
//     rcp<RenderTarget> m_renderTarget;
// };
//
// } // namespace rive::gpu

// Rust has no C++ derived-class base subobject. The first field is the
// intrusive `RefCnt<RenderCanvas>` base, preserving the source base topology;
// the two owning `rcp` fields use ManuallyDrop so explicit Drop releases
// `m_renderTarget`, then `m_renderImage`, without moving the intrusive base
// away from offset zero. `RefCntTarget` supplies the source static-cast
// zero-reference deletion hook through this complete owner.
#[repr(C)]
pub struct RenderCanvas {
    // public RefCnt<RenderCanvas> base class
    pub(crate) base: RefCnt<RenderCanvas>,

    // rcp<RiveRenderImage> m_renderImage;
    m_renderImage: ManuallyDrop<rcp<RiveRenderImage>>,
    // rcp<RenderTarget> m_renderTarget;
    m_renderTarget: ManuallyDrop<rcp<RenderTarget>>,
}

// SAFETY: RenderCanvas' intrusive base is the first `#[repr(C)]` field and
// therefore recovers the complete allocation without pointer adjustment.
unsafe impl RefCntTarget for RenderCanvas {
    // RefCnt's inherited `ref()` operation.
    fn r#ref(&self) {
        self.base.r#ref();
    }

    // RefCnt's inherited `unref()` operation.
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
}

impl RenderCanvas {
    // RenderCanvas(rcp<RiveRenderImage> image, rcp<RenderTarget> target) :
    //     m_renderImage(std::move(image)), m_renderTarget(std::move(target))
    // {}
    /// Constructs the source canvas owner from its two intrusive members.
    ///
    /// # Safety
    /// Both `image` and `target` must be non-null intrusive owners whose
    /// complete allocations remain valid for the lifetime of this canvas.
    /// The source `width`, `height`, and product borrow adapters dereference
    /// those members without a null branch, exactly as the C++ constructor's
    /// later accessors do.
    pub unsafe fn new(image: rcp<RiveRenderImage>, target: rcp<RenderTarget>) -> Self {
        Self {
            base: RefCnt::new(),
            m_renderImage: ManuallyDrop::new(image),
            m_renderTarget: ManuallyDrop::new(target),
        }
    }

    // uint32_t width() const { return m_renderTarget->width(); }
    pub fn width(&self) -> u32 {
        // The source unconditionally dereferences m_renderTarget. Keep that
        // null-dereference quirk rather than adding a source-level guard.
        unsafe { (&*self.m_renderTarget.get()).width() }
    }

    // uint32_t height() const { return m_renderTarget->height(); }
    pub fn height(&self) -> u32 {
        // The source unconditionally dereferences m_renderTarget. Keep that
        // null-dereference quirk rather than adding a source-level guard.
        unsafe { (&*self.m_renderTarget.get()).height() }
    }

    // // Use as a RenderImage for compositing into Rive draws.
    // RiveRenderImage* renderImage() { return m_renderImage.get(); }
    pub fn renderImage(&mut self) -> *mut RiveRenderImage {
        self.m_renderImage.get()
    }

    /// Product-adapter borrow of the exact image member. The returned borrow
    /// cannot outlive this canvas's owning `rcp`, unlike the source raw-pointer
    /// accessor above.
    pub(crate) fn render_image_ref(&self) -> &RiveRenderImage {
        // SAFETY: the source constructor requires a nonnull image owner and
        // this canvas retains it for the complete borrow.
        unsafe { &*self.m_renderImage.get() }
    }

    /// Product-adapter copy of the exact `m_renderImage` intrusive owner.
    /// This is the source `rcp` copy operation, not a texture re-adoption.
    pub(crate) fn ref_render_image(&self) -> rcp<RiveRenderImage> {
        rcp::copy_ctor(&self.m_renderImage)
    }

    // // Use as a RenderTarget for rendering into this texture.
    // RenderTarget* renderTarget() { return m_renderTarget.get(); }
    pub fn renderTarget(&mut self) -> *mut RenderTarget {
        self.m_renderTarget.get()
    }

    /// Product-adapter borrow of the exact target member. This deliberately
    /// does not publish the backend target or any of its registry handles.
    pub(crate) fn render_target_ref(&self) -> &RenderTarget {
        // SAFETY: the source constructor requires a nonnull target owner and
        // this canvas retains it for the complete borrow.
        unsafe { &*self.m_renderTarget.get() }
    }
}

impl Drop for RenderCanvas {
    fn drop(&mut self) {
        // C++ destroys members in reverse declaration order, then bases.
        // The intrusive base must remain at offset zero, so explicit member
        // drops preserve both requirements.
        unsafe {
            ManuallyDrop::drop(&mut self.m_renderTarget);
            ManuallyDrop::drop(&mut self.m_renderImage);
        }
    }
}
