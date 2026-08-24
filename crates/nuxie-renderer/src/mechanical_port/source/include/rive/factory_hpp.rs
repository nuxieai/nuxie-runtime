/*
 * Copyright 2022 Rive
 */

// Mechanical translation of the complete pinned source header
// include/rive/factory.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// /*
//  * Copyright 2022 Rive
//  */
//
// #ifndef _RIVE_FACTORY_HPP_
// #define _RIVE_FACTORY_HPP_
//
// #include "rive/renderer.hpp"
// #include "rive/text_engine.hpp"
// #include "rive/audio/audio_source.hpp"
// #include "rive/refcnt.hpp"
// #include "rive/span.hpp"
// #include "rive/math/aabb.hpp"
//
// #include <stdio.h>
// #include <cstdint>
//
// namespace rive
// {
//
// class RawPath;
// namespace ore
// {
// class Context;
// }
//
// class Factory
// {
// public:
//     Factory() {}
//     virtual ~Factory() {}
//
//     virtual rcp<RenderBuffer> makeRenderBuffer(RenderBufferType,
//                                                RenderBufferFlags,
//                                                size_t sizeInBytes) = 0;
//
//     virtual rcp<RenderShader> makeLinearGradient(
//         float sx,
//         float sy,
//         float ex,
//         float ey,
//         const ColorInt colors[], // [count]
//         const float stops[],     // [count]
//         size_t count) = 0;
//
//     virtual rcp<RenderShader> makeRadialGradient(
//         float cx,
//         float cy,
//         float radius,
//         const ColorInt colors[], // [count]
//         const float stops[],     // [count]
//         size_t count) = 0;
//
//     // Returns a full-formed RenderPath -- can be treated as immutable
//     // This call might swap out the arrays backing the points and verbs in the
//     // given RawPath, so the caller can expect it to be in an undefined state
//     // upon return.
//     virtual rcp<RenderPath> makeRenderPath(RawPath&, FillRule) = 0;
//
//     // Deprecated -- working to make RenderPath's immutable
//     virtual rcp<RenderPath> makeEmptyRenderPath() = 0;
//
//     virtual rcp<RenderPaint> makeRenderPaint() = 0;
//
//     virtual rcp<RenderImage> decodeImage(Span<const uint8_t>) = 0;
//
//     // GPU ore context, when this factory is backed by a RenderContext.
//     // Null for non-GPU factories. Kept last in the virtual section to avoid
//     // shifting existing vtable slots.
//     virtual ore::Context* ore() { return nullptr; }
//
//     rcp<Font> decodeFont(Span<const uint8_t>);
//
//     rcp<AudioSource> decodeAudio(Span<const uint8_t>);
//
//     // Non-virtual helpers
//
//     rcp<RenderPath> makeRenderPath(const AABB&);
//
// };
//
// } // namespace rive
// #endif

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::sync::Arc;

use nuxie_render_api::{
    Aabb, AudioDecodeError, AudioSource, ColorInt, DecodedFont, FillRule, FontDecodeError, RawPath,
};

use super::refcnt_hpp::{rcp, RefCnt, RefCntTarget};
use super::renderer_hpp::{
    RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint, RenderPath,
    RenderShader,
};

/// Owning spelling of the source-polymorphic `rive::ore::Context`. Each
/// concrete allocation stays boxed at its original address, matching the
/// source `unique_ptr<ore::Context>` when multiple native backends compile in
/// the same crate.
#[cfg(any(
    feature = "native-ore-metal-experimental",
    feature = "native-ore-vulkan-experimental",
    feature = "native-webgpu-experimental",
    feature = "ore-gl"
))]
pub enum OreContext {
    #[cfg(feature = "native-ore-metal-experimental")]
    Metal(Box<nuxie_ore_metal::metal::context::ContextMetal>),
    #[cfg(feature = "native-ore-vulkan-experimental")]
    Vulkan(Box<crate::mechanical_port::vulkan::ContextVulkan>),
    #[cfg(feature = "native-webgpu-experimental")]
    WGPU(Box<crate::mechanical_port::webgpu::ContextWGPU>),
    #[cfg(feature = "ore-gl")]
    GL(Box<crate::mechanical_port::webgl2::ContextGL>),
}
#[cfg(not(any(
    feature = "native-ore-metal-experimental",
    feature = "native-ore-vulkan-experimental",
    feature = "native-webgpu-experimental",
    feature = "ore-gl"
)))]
pub enum OreContext {}

/// Source class identity. Factory has no data members; Rust dispatch lives in
/// FactoryContract, while this embedded base preserves the authored hierarchy.
#[repr(C)]
pub struct Factory {
    // C++ Factory is polymorphic even though it has no authored data members;
    // this raw slot is the honest non-ZST representation of its vptr-bearing
    // base subobject. Concrete owners install their dispatch through the Rust
    // FactoryContract implementation rather than fabricating a duplicate slot.
    pub(crate) vtable: unsafe fn(*mut Factory),
}

// The source object is polymorphic even though it has no authored data
// members. Keep a real installed dispatch slot in every base subobject; a
// null pointer would be padding, not the source vptr-bearing representation.
unsafe fn factory_virtual_slot(_self: *mut Factory) {}

impl Default for Factory {
    fn default() -> Self {
        Self {
            vtable: factory_virtual_slot,
        }
    }
}

pub trait FactoryAccess {
    fn factory(&self) -> &Factory;
    fn factoryMut(&mut self) -> &mut Factory;
}

/// Source `Factory`'s abstract virtual surface.  Pure virtual members remain
/// required contract methods; `ore()` retains the source's default nullptr and
/// its last-vtable-slot ordering. The virtual methods retain the source raw
/// pointer/count ABI. Slice adapters live on concrete product owners and
/// validate lengths before crossing this boundary.
pub trait FactoryContract: FactoryAccess {
    fn makeRenderBuffer(
        &mut self,
        bufferType: RenderBufferType,
        bufferFlags: RenderBufferFlags,
        sizeInBytes: usize,
    ) -> rcp<RenderBuffer>;
    unsafe fn makeLinearGradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: *const ColorInt,
        stops: *const f32,
        count: usize,
    ) -> rcp<RenderShader>;
    unsafe fn makeRadialGradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: *const ColorInt,
        stops: *const f32,
        count: usize,
    ) -> rcp<RenderShader>;
    fn makeRenderPath(&mut self, rawPath: &mut RawPath, fillRule: FillRule) -> rcp<RenderPath>;
    fn makeEmptyRenderPath(&mut self) -> rcp<RenderPath>;
    fn makeRenderPaint(&mut self) -> rcp<RenderPaint>;
    unsafe fn decodeImage(&mut self, encoded: *const u8, size: usize) -> rcp<RenderImage>;

    /// `virtual ore::Context* ore() { return nullptr; }` remains a typed
    /// nullable raw pointer until the ORE owner is wired to this factory.
    unsafe fn ore(&mut self) -> *mut OreContext {
        core::ptr::null_mut()
    }
}

/// Public high-level adapters remain outside the source virtual trait. They
/// intentionally expose Result/Arc values while the source-shaped helpers
/// below preserve nullable intrusive ownership.
pub fn decodeFont(encoded: &[u8]) -> Result<DecodedFont, FontDecodeError> {
    nuxie_render_api::decode_font_bytes(encoded)
}

pub fn decodeAudio(encoded: &[u8]) -> Result<Arc<AudioSource>, AudioDecodeError> {
    nuxie_render_api::decode_audio_bytes(encoded)
}

pub fn makeRenderPathFromAABB<F: FactoryContract + ?Sized>(
    factory: &mut F,
    bounds: Aabb,
) -> rcp<RenderPath> {
    let mut rawPath = RawPath::new();
    rawPath.add_rect(bounds);
    factory.makeRenderPath(&mut rawPath, FillRule::NonZero)
}

pub fn decodeImage<F: FactoryContract + ?Sized>(
    factory: &mut F,
    encoded: &[u8],
) -> rcp<RenderImage> {
    // SAFETY: the slice pointer/count pair is valid for the duration of the
    // source decode call; the implementation copies or consumes it before
    // returning and does not retain the borrowed bytes.
    unsafe { factory.decodeImage(encoded.as_ptr(), encoded.len()) }
}

/// Source-shaped nullable intrusive Font owner. The feature-specific decoder
/// may return null; high-level callers should use `decodeFont` above.
pub type SourceFont = crate::mechanical_port::source::src::renderer_cpp::Font;

/// AudioSource is feature-owned outside this renderer closure. Keep its
/// source intrusive/null ABI explicit without pretending the high-level Arc
/// audio value is a C++ RefCnt allocation.
#[repr(C)]
pub struct SourceAudioSource {
    pub(crate) base: RefCnt<SourceAudioSource>,
}

unsafe impl RefCntTarget for SourceAudioSource {
    fn r#ref(&self) {
        self.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
}

pub fn decodeFontSource(_encoded: *const u8, _size: usize) -> rcp<SourceFont> {
    // WITH_RIVE_TEXT is not an active owner in this mechanical closure;
    // preserve the pinned feature-null return instead of manufacturing a Font.
    rcp::new()
}

pub fn decodeAudioSource(_encoded: *const u8, _size: usize) -> rcp<SourceAudioSource> {
    // WITH_RIVE_AUDIO is likewise outside this renderer owner.
    rcp::new()
}
