/*
 * Copyright 2022 Rive
 */

// Mechanical translation of the complete pinned source header
// include/rive/renderer.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// The complete source is retained below in declaration order. The Rust
// declarations following it keep the source names, field order, defaults,
// nullable links, intrusive owners, configuration condition, and virtual
// contracts visible to later mechanical translations.

// /*
//  * Copyright 2022 Rive
//  */
//
// #ifndef _RIVE_RENDERER_HPP_
// #define _RIVE_RENDERER_HPP_
//
// #include "rive/enums.hpp"
// #include "rive/shapes/paint/color.hpp"
// #include "rive/command_path.hpp"
// #include "rive/layout.hpp"
// #include "rive/refcnt.hpp"
// #include "rive/math/aabb.hpp"
// #include "rive/math/mat2d.hpp"
// #include "rive/shapes/paint/blend_mode.hpp"
// #include "rive/shapes/paint/image_sampler.hpp"
// #include "rive/shapes/paint/stroke_cap.hpp"
// #include "rive/shapes/paint/stroke_join.hpp"
// #include "utils/lite_rtti.hpp"
// #include "rive/math/raw_path.hpp"
// #include <stdio.h>
// #include <cstdint>
//
// namespace rive
// {
// class Vec2D;
//
// // Helper that computes a matrix to "align" content (source) to fit inside frame
// // (destination).
// Mat2D computeAlignment(Fit,
//                        Alignment,
//                        const AABB& frame,
//                        const AABB& content,
//                        const float scaleFactor = 1.0f);
//
// enum class RenderBufferType
// {
//     index,
//     vertex,
// };
//
// enum class RenderBufferFlags
// {
//     none = 0,
//     mappedOnceAtInitialization =
//         1 << 0, // The client will map the buffer exactly one time, before
//                 // rendering, and will never update it again.
// };
//
// class RenderBuffer : public RefCnt<RenderBuffer>,
//                      public ENABLE_LITE_RTTI(RenderBuffer)
// {
// public:
//     RenderBuffer(RenderBufferType, RenderBufferFlags, size_t sizeInBytes);
//     virtual ~RenderBuffer();
//
//     RenderBufferType type() const { return m_type; }
//     RenderBufferFlags flags() const { return m_flags; }
//     size_t sizeInBytes() const { return m_sizeInBytes; }
//
//     void* map();
//     void unmap();
//
// protected:
//     virtual void* onMap() = 0;
//     virtual void onUnmap() = 0;
//
//     // Unset the dirty flag, and return whether it had been set.
//     bool checkAndResetDirty()
//     {
//         assert(m_mapCount == m_unmapCount); // Don't call this while mapped.
//         if (m_dirty)
//         {
//             m_dirty = false;
//             return true;
//         }
//         return false;
//     }
//
// private:
//     const RenderBufferType m_type;
//     const RenderBufferFlags m_flags;
//     const size_t m_sizeInBytes;
//     bool m_dirty = false;
//     RIVE_DEBUG_CODE(size_t m_mapCount = 0;)
//     RIVE_DEBUG_CODE(size_t m_unmapCount = 0;)
// };
//
// enum class RenderPaintStyle
// {
//     stroke,
//     fill
// };
//
// /*
//  *  Base class for Render objects that specify the src colors.
//  *
//  *  Shaders are immutable, and sharable between multiple paints, etc.
//  *
//  *  It is common that a shader may be created with a 'localMatrix'. If this is
//  *  not null, then it is applied to the shader's domain before the Renderer's
//  * CTM.
//  */
// class RenderShader : public RefCnt<RenderShader>,
//                      public ENABLE_LITE_RTTI(RenderShader)
// {
// public:
//     RenderShader();
//     virtual ~RenderShader();
// };
//
// class RenderPaint : public RefCnt<RenderPaint>,
//                     public ENABLE_LITE_RTTI(RenderPaint)
// {
// public:
//     RenderPaint();
//     virtual ~RenderPaint();
//
//     virtual void style(RenderPaintStyle style) = 0;
//     virtual void color(ColorInt value) = 0;
//     virtual void thickness(float value) = 0;
//     virtual void join(StrokeJoin value) = 0;
//     virtual void cap(StrokeCap value) = 0;
//     virtual void feather(float value) {} // Not supported on all renderers.
//     virtual void blendMode(BlendMode value) = 0;
//     virtual void shader(rcp<RenderShader>) = 0;
//     virtual void invalidateStroke() = 0;
// };
//
// #if defined(__EMSCRIPTEN__)
// class RenderImageDelegate
// {
// public:
//     virtual void decodedAsync() = 0;
// };
// #endif
//
// class RenderImage : public RefCnt<RenderImage>,
//                     public ENABLE_LITE_RTTI(RenderImage)
// {
// protected:
//     int m_Width = 0;
//     int m_Height = 0;
//     Mat2D m_uvTransform;
//
// public:
//     RenderImage();
//     RenderImage(const Mat2D& uvTransform);
//     virtual ~RenderImage();
//
//     int width() const { return m_Width; }
//     int height() const { return m_Height; }
//     const Mat2D& uvTransform() const { return m_uvTransform; }
//
// #if defined(__EMSCRIPTEN__)
//     void delegate(RenderImageDelegate* delegate) { m_delegate = delegate; }
//     void decodedAsync() const
//     {
//         if (m_delegate != nullptr)
//         {
//             m_delegate->decodedAsync();
//         }
//     }
//
// private:
//     RenderImageDelegate* m_delegate = nullptr;
// #endif
// };
//
// class RenderPath : public CommandPath, public ENABLE_LITE_RTTI(RenderPath)
// {
// public:
//     RenderPath();
//     ~RenderPath() override;
//
//     RenderPath* renderPath() override { return this; }
//     const RenderPath* renderPath() const override { return this; }
//
//     void addPath(CommandPath* path, const Mat2D& transform) override
//     {
//         addRenderPath(path->renderPath(), transform);
//     }
//
//     void addPathBackwards(CommandPath* path, const Mat2D& transform)
//     {
//         addRenderPath(path->renderPath(), transform);
//     }
//
//     virtual void addRenderPath(const RenderPath* path,
//                                const Mat2D& transform) = 0;
//     virtual void addRenderPathBackwards(const RenderPath* path,
//                                         const Mat2D& transform)
//     {
//         // No-op on non rive renderer.
//     }
//
//     virtual void addRawPath(const RawPath& path) = 0;
// };
//
// class Renderer
// {
// public:
//     virtual ~Renderer() {}
//     virtual void save() = 0;
//     virtual void restore() = 0;
//     virtual void transform(const Mat2D& transform) = 0;
//     virtual void drawPath(RenderPath* path, RenderPaint* paint) = 0;
//     virtual void clipPath(RenderPath* path) = 0;
//     virtual void drawImage(const RenderImage*,
//                            ImageSampler,
//                            BlendMode,
//                            float opacity) = 0;
//     virtual void drawImageMesh(const RenderImage*,
//                                ImageSampler,
//                                rcp<RenderBuffer> vertices_f32,
//                                rcp<RenderBuffer> uvCoords_f32,
//                                rcp<RenderBuffer> indices_u16,
//                                uint32_t vertexCount,
//                                uint32_t indexCount,
//                                BlendMode,
//                                float opacity) = 0;
//
//     // Modulate the opacity of subsequent draw calls. The opacity is stacked
//     // multiplicatively (e.g., modulateOpacity(0.5) followed by
//     // modulateOpacity(0.2) = 0.1 effective opacity). The modulated opacity is
//     // captured by save() and restored by restore().
//     virtual void modulateOpacity(float opacity) = 0;
//
//     // helpers
//
//     void translate(float x, float y);
//     void scale(float sx, float sy);
//     void rotate(float radians);
//
//     void align(Fit fit,
//                Alignment alignment,
//                const AABB& frame,
//                const AABB& content,
//                const float scaleFactor = 1.0f)
//     {
//         transform(
//             computeAlignment(fit, alignment, frame, content, scaleFactor));
//     }
// };
// } // namespace rive
// #endif

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::c_void;
use core::ptr::NonNull;

// Mapped source owners. The generic rcp and ImageSampler headers are retained
// as neighboring mechanical translations; project value types remain mapped
// by the existing render-api owner rather than redefined here.
use super::super::utils::lite_rtti_hpp::{LiteRttiBase, LiteRttiCastFrom, LiteRttiTypeId};
use super::refcnt_hpp::{rcp, RefCnt, RefCntTarget};
use super::shapes::paint::image_sampler_hpp::ImageSampler;
use crate::mechanical_port::source::src::renderer_cpp::computeAlignment;
#[cfg(any(
    feature = "native-webgpu-experimental",
    feature = "ore-gl"
))]
use nuxie_ore_metal::gpu_resource::{OwnerThreadFinalRelease, OwnerThreadFinalReleaseRoute};
use nuxie_render_api::{
    Aabb as AABB, BlendMode, ColorInt, FillRule, Fit, Mat2D, RawPath, StrokeCap, StrokeJoin, Vec2D,
};

pub type Alignment = Vec2D;

// namespace rive
// {
// class Vec2D;
// The forward declaration is represented by the mapped render-api Vec2D at
// consumers; no second local Vec2D owner is introduced in this header.

// Mat2D computeAlignment(Fit,
//                        Alignment,
//                        const AABB& frame,
//                        const AABB& content,
//                        const float scaleFactor = 1.0f);
// Declaration only: the paired renderer.cpp translation owns the definition.

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderBufferType {
    // index,
    index = 0,
    // vertex,
    vertex = 1,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderBufferFlags {
    // none = 0,
    none = 0,
    // mappedOnceAtInitialization = 1 << 0,
    // The client will map the buffer exactly one time, before rendering, and
    // will never update it again.
    mappedOnceAtInitialization = 1 << 0,
}

// class RenderBuffer : public RefCnt<RenderBuffer>,
//                      public ENABLE_LITE_RTTI(RenderBuffer)
// Rust has no C++ base subobject. RefCnt<RenderBuffer> remains the source
// intrusive owner contract and ENABLE_LITE_RTTI(RenderBuffer) remains the
// checked most-derived RTTI contract; neither base is duplicated as payload
// state in this header.
#[repr(C)]
pub struct RenderBuffer {
    // RefCnt<RenderBuffer> must be offset zero because its zero transition
    // performs the authored base-to-complete-object cast.
    pub(crate) base: RefCnt<RenderBuffer>,
    pub(crate) destroy_complete: unsafe fn(*mut RenderBuffer),
    // Protected pure virtuals are carried by the base object exactly like the
    // C++ vptr. Public map()/unmap() dispatch these slots on the complete
    // authored owner; callers never supply a second hooks object.
    pub(crate) on_map: unsafe fn(*mut RenderBuffer) -> *mut c_void,
    pub(crate) on_unmap: unsafe fn(*mut RenderBuffer),
    // ENABLE_LITE_RTTI(RenderBuffer)::m_liteTypeId.
    pub(crate) m_liteTypeId: u32,
    // const RenderBufferType m_type;
    // Immutable copied value; construction publishes it before any backend
    // allocation and no later source write retargets it.
    pub(crate) m_type: RenderBufferType,
    // const RenderBufferFlags m_flags;
    // Immutable copied bit flag controlling the one-time map contract.
    pub(crate) m_flags: RenderBufferFlags,
    // const size_t m_sizeInBytes;
    // Exact byte-size domain; no inferred or saturating length is introduced.
    pub(crate) m_sizeInBytes: usize,
    // bool m_dirty = false;
    pub(crate) m_dirty: bool,
    // RIVE_DEBUG_CODE(size_t m_mapCount = 0;)
    #[cfg(debug_assertions)]
    pub(crate) m_mapCount: usize,
    // RIVE_DEBUG_CODE(size_t m_unmapCount = 0;)
    #[cfg(debug_assertions)]
    pub(crate) m_unmapCount: usize,

    // Rust-only safety sidecar after the complete source prefix. A WebGL
    // RenderBuffer is erased behind `rcp<RenderBuffer>`, whose atomic last
    // release may occur on a worker. This weak route returns complete-object
    // destruction to the GL owner thread without retaining the context.
    #[cfg(any(
        feature = "native-webgpu-experimental",
        feature = "ore-gl"
    ))]
    pub(crate) rust_final_release_route: Option<OwnerThreadFinalReleaseRoute>,
}

impl LiteRttiBase for RenderBuffer {
    fn liteTypeID(&self) -> u32 {
        RenderBuffer::liteTypeID(self)
    }
    fn setLiteTypeID(&mut self, id: u32) {
        self.m_liteTypeId = id;
    }
}

// SAFETY: the intrusive base is at offset zero and `destroy_complete` retains
// the source virtual-destructor provenance for the complete allocation.
unsafe impl RefCntTarget for RenderBuffer {
    fn r#ref(&self) {
        self.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        let ptr = ptr.cast_mut();
        #[cfg(any(
            feature = "native-webgpu-experimental",
            feature = "ore-gl"
        ))]
        if let Some(route) = unsafe { &*ptr }.rust_final_release_route.as_ref() {
            unsafe fn destroy_on_owner_thread(payload: usize) {
                let ptr = payload as *mut RenderBuffer;
                unsafe { ((*ptr).destroy_complete)(ptr) };
            }
            let release =
                unsafe { OwnerThreadFinalRelease::new(ptr as usize, destroy_on_owner_thread) };
            // Once the route is closed, the allocation is deliberately
            // quarantined. Running the concrete destructor on this arbitrary
            // releasing thread could touch Rc-backed GL state.
            let _ = route.release_or_defer(release);
            return;
        }
        unsafe { ((*ptr).destroy_complete)(ptr) };
    }
}

impl RenderBuffer {
    // RenderBuffer(RenderBufferType, RenderBufferFlags, size_t sizeInBytes);
    // virtual ~RenderBuffer();

    pub fn liteTypeID(&self) -> u32 {
        self.m_liteTypeId
    }

    // RenderBufferType type() const { return m_type; }
    pub fn r#type(&self) -> RenderBufferType {
        self.m_type
    }

    // RenderBufferFlags flags() const { return m_flags; }
    pub fn flags(&self) -> RenderBufferFlags {
        self.m_flags
    }

    // size_t sizeInBytes() const { return m_sizeInBytes; }
    pub fn sizeInBytes(&self) -> usize {
        self.m_sizeInBytes
    }

    #[cfg(any(
        feature = "native-webgpu-experimental",
        feature = "ore-gl"
    ))]
    pub(crate) fn install_owner_thread_final_release_route(
        &mut self,
        route: OwnerThreadFinalReleaseRoute,
    ) {
        assert!(
            self.rust_final_release_route.is_none(),
            "RenderBuffer accepts one owner-thread final-release route"
        );
        self.rust_final_release_route = Some(route);
    }

    // void* map();
    // void unmap();
    // Definitions belong to src/renderer.cpp and must retain the source
    // mapped-once assertion, dirty publication, and onMap/onUnmap dispatch.

    // protected:
    // virtual void* onMap() = 0;
    // virtual void onUnmap() = 0;

    // bool checkAndResetDirty()
    // {
    //     assert(m_mapCount == m_unmapCount); // Don't call this while mapped.
    //     if (m_dirty)
    //     {
    //         m_dirty = false;
    //         return true;
    //     }
    //     return false;
    // }
    pub(crate) fn checkAndResetDirty(&mut self) -> bool {
        #[cfg(debug_assertions)]
        assert!(self.m_mapCount == self.m_unmapCount); // Don't call this while mapped.
        if self.m_dirty {
            self.m_dirty = false;
            return true;
        }
        false
    }
}

// The source's protected pure-virtual slots remain an explicit implementation
// contract for concrete backend owners. The public/nonvirtual map surface is
// defined by the paired renderer.cpp translation and dispatches through the
// slots stored in that same RenderBuffer base object.
pub trait RenderBufferContract {
    fn onMap(&mut self) -> *mut c_void;
    fn onUnmap(&mut self);
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderPaintStyle {
    // stroke,
    stroke = 0,
    // fill
    fill = 1,
}

// /*
//  *  Base class for Render objects that specify the src colors.
//  *
//  *  Shaders are immutable, and sharable between multiple paints, etc.
//  *
//  *  It is common that a shader may be created with a 'localMatrix'. If this
//  *  is not null, then it is applied to the shader's domain before the
//  *  Renderer's CTM.
//  */
// class RenderShader : public RefCnt<RenderShader>,
//                      public ENABLE_LITE_RTTI(RenderShader)
#[repr(C)]
pub struct RenderShader {
    pub(crate) base: RefCnt<RenderShader>,
    pub(crate) destroy_complete: unsafe fn(*mut RenderShader),
    pub(crate) m_liteTypeId: u32,
}

impl LiteRttiBase for RenderShader {
    fn liteTypeID(&self) -> u32 {
        RenderShader::liteTypeID(self)
    }
    fn setLiteTypeID(&mut self, id: u32) {
        self.m_liteTypeId = id;
    }
}

impl LiteRttiTypeId for RenderShader {
    const LITE_RTTI_TYPE_ID: u32 = super::super::utils::lite_rtti_hpp::CONST_ID("RenderShader");
}
impl LiteRttiCastFrom<RenderShader> for RenderShader {
    unsafe fn from_base(base: *mut RenderShader) -> *mut Self {
        base
    }
}

// SAFETY: the intrusive base is at offset zero and complete-object destruction
// is selected by the installed source-shaped destructor slot.
unsafe impl RefCntTarget for RenderShader {
    fn r#ref(&self) {
        self.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { ((*ptr).destroy_complete)(ptr.cast_mut()) };
    }
}

impl RenderShader {
    // RenderShader();
    // virtual ~RenderShader();
    pub fn liteTypeID(&self) -> u32 {
        self.m_liteTypeId
    }
}

// class RenderPaint : public RefCnt<RenderPaint>,
//                     public ENABLE_LITE_RTTI(RenderPaint)
// The source bases remain documented source contracts; the paint has no
// additional data members in the pinned header.
#[repr(C)]
pub struct RenderPaint {
    pub(crate) base: RefCnt<RenderPaint>,
    pub(crate) destroy_complete: unsafe fn(*mut RenderPaint),
    pub(crate) m_liteTypeId: u32,
}

impl LiteRttiBase for RenderPaint {
    fn liteTypeID(&self) -> u32 {
        RenderPaint::liteTypeID(self)
    }
    fn setLiteTypeID(&mut self, id: u32) {
        self.m_liteTypeId = id;
    }
}

impl LiteRttiTypeId for RenderPaint {
    const LITE_RTTI_TYPE_ID: u32 = super::super::utils::lite_rtti_hpp::CONST_ID("RenderPaint");
}
impl LiteRttiCastFrom<RenderPaint> for RenderPaint {
    unsafe fn from_base(base: *mut RenderPaint) -> *mut Self {
        base
    }
}

// SAFETY: the intrusive base is at offset zero and complete-object destruction
// is selected by the installed source-shaped destructor slot.
unsafe impl RefCntTarget for RenderPaint {
    fn r#ref(&self) {
        self.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { ((*ptr).destroy_complete)(ptr.cast_mut()) };
    }
}

impl RenderPaint {
    // RenderPaint();
    // virtual ~RenderPaint();
    pub fn liteTypeID(&self) -> u32 {
        self.m_liteTypeId
    }
}

// virtual void style(RenderPaintStyle style) = 0;
// virtual void color(ColorInt value) = 0;
// virtual void thickness(float value) = 0;
// virtual void join(StrokeJoin value) = 0;
// virtual void cap(StrokeCap value) = 0;
// virtual void feather(float value) {} // Not supported on all renderers.
// virtual void blendMode(BlendMode value) = 0;
// virtual void shader(rcp<RenderShader>) = 0;
// virtual void invalidateStroke() = 0;
pub trait RenderPaintContract {
    fn style(&mut self, style: RenderPaintStyle);
    fn color(&mut self, value: ColorInt);
    fn thickness(&mut self, value: f32);
    fn join(&mut self, value: StrokeJoin);
    fn cap(&mut self, value: StrokeCap);
    fn feather(&mut self, value: f32) {}
    fn blendMode(&mut self, value: BlendMode);
    // rcp<RenderShader> is an intrusive owning transfer, not a borrowed link.
    unsafe fn shader(&mut self, shader: rcp<RenderShader>);
    fn invalidateStroke(&mut self);
}

// #if defined(__EMSCRIPTEN__)
#[cfg(target_os = "emscripten")]
#[repr(C)]
pub struct RenderImageDelegateVTable {
    pub decodedAsync: unsafe extern "C" fn(*mut c_void),
}

#[cfg(target_os = "emscripten")]
#[repr(C)]
pub struct RenderImageDelegate {
    pub vtable: *const RenderImageDelegateVTable,
}
// #endif

// class RenderImage : public RefCnt<RenderImage>,
//                     public ENABLE_LITE_RTTI(RenderImage)
// The source intrusive and RTTI bases are represented as source contracts;
// the protected payload fields retain declaration order and source defaults.
#[repr(C)]
pub struct RenderImage {
    pub(crate) base: RefCnt<RenderImage>,
    pub(crate) destroy_complete: unsafe fn(*mut RenderImage),
    pub(crate) m_liteTypeId: u32,
    // protected:
    // int m_Width = 0;
    // The mechanical field spelling follows the field-authority path for the
    // source's capitalized member while preserving its authored i32 width.
    pub(crate) m__width: i32,
    // int m_Height = 0;
    pub(crate) m__height: i32,
    // Mat2D m_uvTransform;
    pub(crate) m_uv_transform: Mat2D,
    // #if defined(__EMSCRIPTEN__)
    #[cfg(target_os = "emscripten")]
    // RenderImageDelegate* m_delegate = nullptr;
    // This is a nullable non-owning raw-pointer link; it never deletes or
    // retains the delegate and is represented by the authority's exact
    // Option<NonNull<RenderImageDelegate>> shape.
    m_delegate: Option<NonNull<RenderImageDelegate>>,
    // #endif
}

impl LiteRttiBase for RenderImage {
    fn liteTypeID(&self) -> u32 {
        RenderImage::liteTypeID(self)
    }
    fn setLiteTypeID(&mut self, id: u32) {
        self.m_liteTypeId = id;
    }
}

// SAFETY: the intrusive base is at offset zero and complete-object destruction
// is selected by the installed source-shaped destructor slot.
unsafe impl RefCntTarget for RenderImage {
    fn r#ref(&self) {
        self.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { ((*ptr).destroy_complete)(ptr.cast_mut()) };
    }
}

impl RenderImage {
    // public:
    // RenderImage();
    // RenderImage(const Mat2D& uvTransform);
    // virtual ~RenderImage();

    pub fn liteTypeID(&self) -> u32 {
        self.m_liteTypeId
    }

    // int width() const { return m_Width; }
    pub fn width(&self) -> i32 {
        self.m__width
    }

    // int height() const { return m_Height; }
    pub fn height(&self) -> i32 {
        self.m__height
    }

    // const Mat2D& uvTransform() const { return m_uvTransform; }
    pub fn uvTransform(&self) -> &Mat2D {
        &self.m_uv_transform
    }

    // #if defined(__EMSCRIPTEN__)
    #[cfg(target_os = "emscripten")]
    // void delegate(RenderImageDelegate* delegate) { m_delegate = delegate; }
    /// # Safety
    /// The non-owning machine pointer must remain live through every later
    /// decodedAsync call, matching the source delegate contract.
    pub unsafe fn delegate(&mut self, delegate: Option<NonNull<RenderImageDelegate>>) {
        self.m_delegate = delegate;
    }

    // void decodedAsync() const
    // {
    //     if (m_delegate != nullptr)
    //     {
    //         m_delegate->decodedAsync();
    //     }
    // }
    #[cfg(target_os = "emscripten")]
    pub fn decodedAsync(&self) {
        if let Some(delegate) = self.m_delegate {
            unsafe {
                let object = delegate.as_ptr();
                ((*(*object).vtable).decodedAsync)(object.cast());
            }
        }
    }
    // #endif
}

// The authored CommandPath base owns intrusive lifetime and virtual dispatch.
// Its callback slots are installed for the complete offset-zero RenderPath
// backend owner, preserving calls made through CommandPath*.
#[repr(C)]
pub struct CommandPath {
    base: RefCnt<CommandPath>,
    destroy_complete: unsafe fn(*mut CommandPath),
    rewind_slot: unsafe fn(*mut CommandPath),
    fill_rule_slot: unsafe fn(*mut CommandPath, FillRule),
    add_path_slot: unsafe fn(*mut CommandPath, *mut CommandPath, &Mat2D),
    move_to_slot: unsafe fn(*mut CommandPath, f32, f32),
    line_to_slot: unsafe fn(*mut CommandPath, f32, f32),
    cubic_to_slot: unsafe fn(*mut CommandPath, f32, f32, f32, f32, f32, f32),
    close_slot: unsafe fn(*mut CommandPath),
    render_path_slot: unsafe fn(*mut CommandPath) -> *mut RenderPath,
    render_path_const_slot: unsafe fn(*const CommandPath) -> *const RenderPath,
}

unsafe impl RefCntTarget for CommandPath {
    fn r#ref(&self) {
        self.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { ((*ptr).destroy_complete)(ptr.cast_mut()) };
    }
}

impl CommandPath {
    /// # Safety
    /// `Owner` must contain RenderPath and this nested CommandPath base at
    /// offset zero for its complete allocation lifetime.
    unsafe fn new_for_render_path_owner<Owner: RenderPathContract + LiteRttiTypeId>() -> Self {
        unsafe fn destroy<Owner>(p: *mut CommandPath) {
            unsafe { drop(Box::from_raw(p.cast::<Owner>())) }
        }
        unsafe fn rewind<Owner: RenderPathContract>(p: *mut CommandPath) {
            RenderPathContract::rewind(unsafe { &mut *p.cast::<Owner>() })
        }
        unsafe fn fill<Owner: RenderPathContract>(p: *mut CommandPath, v: FillRule) {
            RenderPathContract::fillRule(unsafe { &mut *p.cast::<Owner>() }, v)
        }
        unsafe fn add<Owner: RenderPathContract>(
            p: *mut CommandPath,
            o: *mut CommandPath,
            m: &Mat2D,
        ) {
            let rendered = unsafe { (&mut *o).renderPath() };
            unsafe { RenderPathContract::addRenderPath(&mut *p.cast::<Owner>(), rendered, m) }
        }
        unsafe fn mov<Owner: RenderPathContract>(p: *mut CommandPath, x: f32, y: f32) {
            RenderPathContract::moveTo(unsafe { &mut *p.cast::<Owner>() }, x, y)
        }
        unsafe fn line<Owner: RenderPathContract>(p: *mut CommandPath, x: f32, y: f32) {
            RenderPathContract::lineTo(unsafe { &mut *p.cast::<Owner>() }, x, y)
        }
        unsafe fn cubic<Owner: RenderPathContract>(
            p: *mut CommandPath,
            a: f32,
            b: f32,
            c: f32,
            d: f32,
            e: f32,
            f: f32,
        ) {
            RenderPathContract::cubicTo(unsafe { &mut *p.cast::<Owner>() }, a, b, c, d, e, f)
        }
        unsafe fn close<Owner: RenderPathContract>(p: *mut CommandPath) {
            RenderPathContract::close(unsafe { &mut *p.cast::<Owner>() })
        }
        unsafe fn render<Owner: RenderPathContract>(p: *mut CommandPath) -> *mut RenderPath {
            p.cast()
        }
        unsafe fn render_const<Owner: RenderPathContract>(
            p: *const CommandPath,
        ) -> *const RenderPath {
            p.cast()
        }
        Self {
            base: RefCnt::new(),
            destroy_complete: destroy::<Owner>,
            rewind_slot: rewind::<Owner>,
            fill_rule_slot: fill::<Owner>,
            add_path_slot: add::<Owner>,
            move_to_slot: mov::<Owner>,
            line_to_slot: line::<Owner>,
            cubic_to_slot: cubic::<Owner>,
            close_slot: close::<Owner>,
            render_path_slot: render::<Owner>,
            render_path_const_slot: render_const::<Owner>,
        }
    }
    pub fn rewind(&mut self) {
        unsafe { (self.rewind_slot)(self) }
    }
    pub fn fillRule(&mut self, v: FillRule) {
        unsafe { (self.fill_rule_slot)(self, v) }
    }
    pub unsafe fn addPath(&mut self, p: *mut CommandPath, m: &Mat2D) {
        unsafe { (self.add_path_slot)(self, p, m) }
    }
    pub fn moveTo(&mut self, x: f32, y: f32) {
        unsafe { (self.move_to_slot)(self, x, y) }
    }
    pub fn lineTo(&mut self, x: f32, y: f32) {
        unsafe { (self.line_to_slot)(self, x, y) }
    }
    pub fn cubicTo(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        unsafe { (self.cubic_to_slot)(self, a, b, c, d, e, f) }
    }
    pub fn close(&mut self) {
        unsafe { (self.close_slot)(self) }
    }
    pub fn renderPath(&mut self) -> *mut RenderPath {
        unsafe { (self.render_path_slot)(self) }
    }
    pub fn renderPath_const(&self) -> *const RenderPath {
        unsafe { (self.render_path_const_slot)(self) }
    }
    pub fn addRect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.moveTo(x, y);
        self.lineTo(x + w, y);
        self.lineTo(x + w, y + h);
        self.lineTo(x, y + h);
        self.close()
    }
    pub fn r#move(&mut self, v: Vec2D) {
        self.moveTo(v.x, v.y)
    }
    pub fn line(&mut self, v: Vec2D) {
        self.lineTo(v.x, v.y)
    }
    pub fn cubic(&mut self, a: Vec2D, b: Vec2D, c: Vec2D) {
        self.cubicTo(a.x, a.y, b.x, b.y, c.x, c.y)
    }
}

pub trait CommandPathContract {
    fn commandPath(&self) -> &CommandPath;
    fn commandPathMut(&mut self) -> &mut CommandPath;
    fn rewind(&mut self) {
        self.commandPathMut().rewind()
    }
    fn fillRule(&mut self, v: FillRule) {
        self.commandPathMut().fillRule(v)
    }
    unsafe fn addPath(&mut self, p: *mut CommandPath, m: &Mat2D) {
        unsafe { self.commandPathMut().addPath(p, m) }
    }
    fn moveTo(&mut self, x: f32, y: f32) {
        self.commandPathMut().moveTo(x, y)
    }
    fn lineTo(&mut self, x: f32, y: f32) {
        self.commandPathMut().lineTo(x, y)
    }
    fn cubicTo(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        self.commandPathMut().cubicTo(a, b, c, d, e, f)
    }
    fn close(&mut self) {
        self.commandPathMut().close()
    }
    fn renderPath(&mut self) -> *mut RenderPath {
        self.commandPathMut().renderPath()
    }
    fn renderPath_const(&self) -> *const RenderPath {
        self.commandPath().renderPath_const()
    }
}

impl<T: RenderPathContract> CommandPathContract for T {
    fn commandPath(&self) -> &CommandPath {
        unsafe { &*(self as *const T).cast() }
    }
    fn commandPathMut(&mut self) -> &mut CommandPath {
        unsafe { &mut *(self as *mut T).cast() }
    }
}

// class RenderPath : public CommandPath, public ENABLE_LITE_RTTI(RenderPath)
#[repr(C)]
pub struct RenderPath {
    pub(crate) base: CommandPath,
    m_liteTypeId: u32,
}

impl RenderPath {
    /// # Safety
    /// `Owner` must be the complete offset-zero derived RenderPath allocation.
    pub unsafe fn new_for_owner<Owner: RenderPathContract + LiteRttiTypeId>() -> Self {
        Self {
            base: unsafe { CommandPath::new_for_render_path_owner::<Owner>() },
            m_liteTypeId: Owner::LITE_RTTI_TYPE_ID,
        }
    }
    pub fn liteTypeID(&self) -> u32 {
        self.m_liteTypeId
    }
}

impl LiteRttiBase for RenderPath {
    fn liteTypeID(&self) -> u32 {
        RenderPath::liteTypeID(self)
    }
    fn setLiteTypeID(&mut self, id: u32) {
        self.m_liteTypeId = id
    }
}

impl LiteRttiTypeId for RenderPath {
    const LITE_RTTI_TYPE_ID: u32 = super::super::utils::lite_rtti_hpp::CONST_ID("RenderPath");
}
impl LiteRttiCastFrom<RenderPath> for RenderPath {
    unsafe fn from_base(base: *mut RenderPath) -> *mut Self {
        base
    }
}

unsafe impl RefCntTarget for RenderPath {
    fn r#ref(&self) {
        RefCntTarget::r#ref(&self.base)
    }
    unsafe fn unref(&self) {
        unsafe { RefCntTarget::unref(&self.base) }
    }
}

/// # Safety
/// Implementors must contain the translated `CommandPath` then `RenderPath`
/// bases at offset zero so the inherited source static casts recover the same
/// complete object and destructor slots.
pub unsafe trait RenderPathContract: Sized {
    fn rewind(&mut self);
    fn fillRule(&mut self, value: FillRule);
    fn moveTo(&mut self, x: f32, y: f32);
    fn lineTo(&mut self, x: f32, y: f32);
    fn cubicTo(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32);
    fn close(&mut self);
    // RenderPath();
    // ~RenderPath() override;

    // RenderPath* renderPath() override { return this; }
    fn renderPath(&mut self) -> *mut RenderPath {
        (self as *mut Self).cast()
    }
    // const RenderPath* renderPath() const override { return this; }
    // Rust cannot overload solely on receiver mutability, so the const source
    // overload has a distinct mechanical spelling.
    fn renderPath_const(&self) -> *const RenderPath {
        (self as *const Self).cast()
    }

    // void addPath(CommandPath* path, const Mat2D& transform) override
    // {
    //     addRenderPath(path->renderPath(), transform);
    // }
    unsafe fn addPath(&mut self, path: *mut CommandPath, transform: &Mat2D) {
        // SAFETY: the source dereferences CommandPath* before dispatch; the
        // caller must provide the same live non-null path.
        unsafe { self.addRenderPath((&mut *path).renderPath(), transform) };
    }

    // void addPathBackwards(CommandPath* path, const Mat2D& transform)
    // {
    //     addRenderPath(path->renderPath(), transform);
    // }
    unsafe fn addPathBackwards(&mut self, path: *mut CommandPath, transform: &Mat2D) {
        // SAFETY: this preserves the source raw-pointer dereference contract.
        unsafe { self.addRenderPath((&mut *path).renderPath(), transform) };
    }

    // virtual void addRenderPath(const RenderPath* path,
    //                            const Mat2D& transform) = 0;
    unsafe fn addRenderPath(&mut self, path: *const RenderPath, transform: &Mat2D);

    // virtual void addRenderPathBackwards(const RenderPath* path,
    //                                     const Mat2D& transform)
    // {
    //     // No-op on non rive renderer.
    // }
    unsafe fn addRenderPathBackwards(&mut self, path: *const RenderPath, transform: &Mat2D) {
        let _ = (path, transform);
    }

    // virtual void addRawPath(const RawPath& path) = 0;
    fn addRawPath(&mut self, path: &RawPath);
}

// class Renderer
// The abstract Renderer owns no fields. All resource links passed to its
// drawImageMesh interface retain the source borrowed/owning distinction.
pub struct Renderer;

pub trait RendererContract {
    // virtual ~Renderer() {}
    // virtual void save() = 0;
    fn save(&mut self);
    // virtual void restore() = 0;
    fn restore(&mut self);
    // virtual void transform(const Mat2D& transform) = 0;
    fn transform(&mut self, transform: &Mat2D);
    // virtual void drawPath(RenderPath* path, RenderPaint* paint) = 0;
    unsafe fn drawPath(&mut self, path: *mut RenderPath, paint: *mut RenderPaint);
    // virtual void clipPath(RenderPath* path) = 0;
    unsafe fn clipPath(&mut self, path: *mut RenderPath);
    // virtual void drawImage(const RenderImage*,
    //                        ImageSampler,
    //                        BlendMode,
    //                        float opacity) = 0;
    unsafe fn drawImage(
        &mut self,
        image: *const RenderImage,
        sampler: ImageSampler,
        blendMode: BlendMode,
        opacity: f32,
    );
    // virtual void drawImageMesh(const RenderImage*,
    //                            ImageSampler,
    //                            rcp<RenderBuffer> vertices_f32,
    //                            rcp<RenderBuffer> uvCoords_f32,
    //                            rcp<RenderBuffer> indices_u16,
    //                            uint32_t vertexCount,
    //                            uint32_t indexCount,
    //                            BlendMode,
    //                            float opacity) = 0;
    unsafe fn drawImageMesh(
        &mut self,
        image: *const RenderImage,
        sampler: ImageSampler,
        vertices_f32: rcp<RenderBuffer>,
        uvCoords_f32: rcp<RenderBuffer>,
        indices_u16: rcp<RenderBuffer>,
        vertexCount: u32,
        indexCount: u32,
        blendMode: BlendMode,
        opacity: f32,
    );

    // Modulate the opacity of subsequent draw calls. The opacity is stacked
    // multiplicatively (e.g., modulateOpacity(0.5) followed by
    // modulateOpacity(0.2) = 0.1 effective opacity). The modulated opacity is
    // captured by save() and restored by restore().
    // virtual void modulateOpacity(float opacity) = 0;
    fn modulateOpacity(&mut self, opacity: f32);

    // helpers

    // void translate(float x, float y);
    // void scale(float sx, float sy);
    // void rotate(float radians);
    // Definitions belong to src/renderer.cpp and dispatch through the pure
    // virtual transform() contract exactly once.

    // void align(Fit fit,
    //            Alignment alignment,
    //            const AABB& frame,
    //            const AABB& content,
    //            const float scaleFactor = 1.0f)
    // {
    //     transform(
    //         computeAlignment(fit, alignment, frame, content, scaleFactor));
    // }
    fn align(
        &mut self,
        fit: Fit,
        alignment: Alignment,
        frame: &AABB,
        content: &AABB,
        scaleFactor: f32,
    ) {
        self.transform(&computeAlignment(
            fit,
            alignment,
            frame,
            content,
            scaleFactor,
        ));
    }
}

// } // namespace rive
// #endif
