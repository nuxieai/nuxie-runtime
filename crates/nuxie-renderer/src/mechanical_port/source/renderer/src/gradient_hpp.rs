/*
 * Copyright 2022 Rive
 */

// Mechanical translation of the complete pinned source header
// renderer/src/gradient.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// /*
//  * Copyright 2022 Rive
//  */
//
// #pragma once
//
// #include "rive/renderer/gpu.hpp"
// #include "rive/renderer.hpp"
// #include <array>
//
// namespace rive::gpu
// {
//
// // Copies an array of colors or stops for a gradient.
// // Stores the data locally if there are 4 values or fewer.
// // Spills onto the heap if there are >4 values.
// template <typename T> class GradDataArray
// {
// public:
//     static_assert(std::is_trivial_v<T> && std::is_standard_layout<T>());
//
//     GradDataArray(const T data[], size_t count)
//     {
//         m_data =
//             count <= m_localData.size() ? m_localData.data() : new T[count];
//         memcpy(m_data, data, count * sizeof(T));
//     }
//
//     // Allocate without initializing (caller must fill in data).
//     explicit GradDataArray(size_t count)
//     {
//         m_data =
//             count <= m_localData.size() ? m_localData.data() : new T[count];
//     }
//
//     GradDataArray(GradDataArray&& other)
//     {
//         if (other.m_data == other.m_localData.data())
//         {
//             m_localData = other.m_localData;
//             m_data = m_localData.data();
//         }
//         else
//         {
//             m_data = other.m_data;
//             other.m_data =
//                 other.m_localData.data(); // Don't delete[] other.m_data.
//         }
//     }
//
//     ~GradDataArray()
//     {
//         if (m_data != m_localData.data())
//         {
//             delete[] m_data;
//         }
//     }
//
//     const T* get() const { return m_data; }
//     const T operator[](size_t i) const { return m_data[i]; }
//     T& operator[](size_t i) { return m_data[i]; }
//
// private:
//     std::array<T, 4> m_localData;
//     T* m_data;
// };
//
// // RenderShader implementation for Rive's pixel local storage renderer.
// class Gradient : public LITE_RTTI_OVERRIDE(RenderShader, Gradient)
// {
// public:
//     static rcp<Gradient> MakeLinear(float sx,
//                                     float sy,
//                                     float ex,
//                                     float ey,
//                                     const ColorInt colors[], // [count]
//                                     const float stops[],     // [count]
//                                     size_t count);
//
//     static rcp<Gradient> MakeRadial(float cx,
//                                     float cy,
//                                     float radius,
//                                     const ColorInt colors[], // [count]
//                                     const float stops[],     // [count]
//                                     size_t count);
//
//     PaintType paintType() const { return m_paintType; }
//     const float* coeffs() const { return m_coeffs.data(); }
//     const ColorInt* colors() const { return m_colors.get(); }
//     const float* stops() const { return m_stops.get(); }
//     size_t count() const { return m_count; }
//     bool isOpaque() const;
//
//     // Get or create a modulated variant of this gradient.
//     // Caches the last-used modulated gradient for efficient reuse when the same
//     // opacity is requested multiple times (e.g., multiple draws in one frame).
//     rcp<Gradient> getModulated(float opacity) const;
//
// private:
//     Gradient(PaintType paintType,
//              GradDataArray<ColorInt>&& colors, // [count]
//              GradDataArray<float>&& stops,     // [count]
//              size_t count,
//              float coeffX,
//              float coeffY,
//              float coeffZ) :
//         m_paintType(paintType),
//         m_colors(std::move(colors)),
//         m_stops(std::move(stops)),
//         m_count(count),
//         m_coeffs{coeffX, coeffY, coeffZ}
//     {
//         assert(paintType == gpu::PaintType::linearGradient ||
//                paintType == gpu::PaintType::radialGradient);
//     }
//
//     PaintType m_paintType; // Specifically, linearGradient or radialGradient.
//     GradDataArray<ColorInt> m_colors;
//     GradDataArray<float> m_stops;
//     size_t m_count;
//     std::array<float, 3> m_coeffs;
//     mutable gpu::TriState m_isOpaque = gpu::TriState::unknown;
//
//     // Single-entry cache for last-used modulated gradient
//     mutable rcp<Gradient> m_lastModulatedGradient;
//     mutable float m_lastModulatedOpacity =
//         -1.0f; // -1 as sentinel (valid range is 0..1)
// };
//
// } // namespace rive::gpu

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

use nuxie_render_api::ColorInt;

use crate::mechanical_port::source::include::rive::refcnt_hpp::{
    make_rcp as allocate_rcp, rcp, RefCnt, RefCntTarget,
};
use crate::mechanical_port::source::include::rive::renderer_hpp::RenderShader;
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{
    LiteRttiCastFrom, LiteRttiTypeId, CONST_ID,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;

/// Source-shaped representation of the C++ `GradDataArray<T>`.
///
/// The object is exactly inline storage followed by the active pointer. The
/// source count is retained by Gradient, not by this two-field ABI mirror.
mod grad_data_value {
    pub trait Sealed {}
    impl Sealed for u32 {}
    impl Sealed for f32 {}
}

/// The pinned template is instantiated only for ColorInt (`u32`) and `float`.
/// Closing the Rust equivalent over those two trivial standard-layout values
/// prevents a Drop-bearing T from entering MaybeUninit storage.
pub trait GradDataValue: grad_data_value::Sealed + Copy + Send + Sync + 'static {}
impl GradDataValue for u32 {}
impl GradDataValue for f32 {}

#[repr(C)]
pub struct GradDataArray<T: GradDataValue> {
    // Source order: local inline storage followed by the active data pointer.
    // A null data pointer means the active range is local; this avoids a
    // self-referential pointer that would become stale when the Rust owner is
    // moved. A non-null pointer owns the spilled allocation and is released by
    // Drop, matching C++'s `delete[]` branch.
    m_localData: [MaybeUninit<T>; 4],
    m_data: *mut T,
}

impl<T: GradDataValue> GradDataArray<T> {
    const LOCAL_CAPACITY: usize = 4;

    fn uninitialized_local() -> [MaybeUninit<T>; 4] {
        // SAFETY: an array of `MaybeUninit<T>` may be uninitialized; each slot
        // is written before it is read by the source-shaped API.
        unsafe { MaybeUninit::<[MaybeUninit<T>; 4]>::uninit().assume_init() }
    }

    /// Source `GradDataArray(const T data[], size_t count)`.
    pub unsafe fn from_raw(data: *const T, count: usize) -> Self {
        let mut value = Self {
            m_localData: Self::uninitialized_local(),
            m_data: if count > Self::LOCAL_CAPACITY {
                Self::allocate_spill(count)
            } else {
                core::ptr::null_mut()
            },
        };
        if count != 0 {
            // SAFETY: the source requires a valid `data[0..count]` span for
            // this constructor; the destination has exactly that capacity.
            unsafe { core::ptr::copy_nonoverlapping(data, value.get_mut_ptr(), count) };
        }
        value
    }

    /// Safe slice spelling used by the source factory adapter.
    pub fn from_slice(data: &[T]) -> Self {
        // SAFETY: a Rust slice is a valid source span of exactly `len` values.
        unsafe { Self::from_raw(data.as_ptr(), data.len()) }
    }

    /// Source `explicit GradDataArray(size_t count)`.
    pub unsafe fn uninitialized(count: usize) -> Self {
        Self {
            m_localData: Self::uninitialized_local(),
            m_data: if count > Self::LOCAL_CAPACITY {
                Self::allocate_spill(count)
            } else {
                core::ptr::null_mut()
            },
        }
    }

    fn allocate_spill(count: usize) -> *mut T {
        // Keep the authored two-field object layout while retaining the
        // allocation count in a header immediately before spilled storage.
        // This external metadata is needed only for Rust's allocator on Drop;
        // it is not part of GradDataArray and therefore cannot shift Gradient.
        let (unpad_layout, data_offset) = Layout::new::<usize>()
            .extend(Layout::array::<T>(count).expect("gradient data layout"))
            .expect("gradient data layout");
        let layout = unpad_layout.pad_to_align();
        let base = unsafe { std::alloc::alloc(layout) };
        if base.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        unsafe {
            base.cast::<usize>().write(count);
        }
        unsafe { base.add(data_offset).cast::<T>() }
    }

    fn get_ptr(&self) -> *const T {
        if self.m_data.is_null() {
            self.m_localData.as_ptr().cast::<T>()
        } else {
            self.m_data
        }
    }

    fn get_mut_ptr(&mut self) -> *mut T {
        if self.m_data.is_null() {
            self.m_localData.as_mut_ptr().cast::<T>()
        } else {
            self.m_data
        }
    }

    /// Initializes or overwrites one source slot without forming a reference
    /// to uninitialized T. The caller supplies the source count externally.
    pub unsafe fn write(&mut self, index: usize, value: T) {
        // SAFETY: the destination is MaybeUninit-backed storage with capacity
        // established by the source count constructor; raw write does not read
        // or drop the previous trivial value.
        unsafe { self.get_mut_ptr().add(index).write(value) };
    }

    /// Source `const T* get() const`.
    pub fn get(&self) -> *const T {
        self.get_ptr()
    }

    /// Source `operator[]` read. The caller must prove `index < count` and
    /// that the corresponding source slot has been initialized.
    pub unsafe fn read(&self, index: usize) -> T {
        unsafe { *self.get_ptr().add(index) }
    }

    /// A borrowed view over the source range; the count is Gradient metadata,
    /// matching the C++ object which stores no length in GradDataArray.
    pub unsafe fn as_slice(&self, count: usize) -> &[T] {
        // SAFETY: every element in the source range is initialized before this
        // view is requested; the source API exposes the same raw span.
        unsafe { core::slice::from_raw_parts(self.get_ptr(), count) }
    }
}

impl<T: GradDataValue> GradDataArray<T> {
    /// Releases a spilled source allocation. The C++ move constructor transfers
    /// the active pointer; Rust's moved-from value is not dropped, and the
    /// owning destination's Drop invokes this once. Gradient's explicit
    /// reverse-order release nulls its members before their automatic Drops.
    unsafe fn release(&mut self) {
        if !self.m_data.is_null() {
            let header = unsafe {
                self.m_data
                    .cast::<u8>()
                    .sub(core::mem::size_of::<usize>())
                    .cast::<usize>()
            };
            let count = unsafe { *header };
            let (unpad_layout, _) = Layout::new::<usize>()
                .extend(Layout::array::<T>(count).expect("gradient data layout"))
                .expect("gradient data layout");
            let layout = unpad_layout.pad_to_align();
            unsafe {
                std::alloc::dealloc(header.cast(), layout);
            }
            self.m_data = core::ptr::null_mut();
        }
    }
}

impl<T: GradDataValue> Drop for GradDataArray<T> {
    fn drop(&mut self) {
        // SAFETY: a GradDataArray owns only its optional spilled allocation;
        // release reads the external count header and nulls the pointer.
        unsafe {
            self.release();
        }
    }
}

struct GradientConstructionSeal;

#[repr(C)]
pub struct Gradient {
    // `Gradient` derives from `RenderShader` in the source. Keeping that
    // intrusive/RTTI base at offset zero preserves static_rcp_cast and the
    // source virtual-owner destruction path.
    pub(super) base: RenderShader,
    pub(super) m_paintType: gpu::PaintType,
    pub(super) m_colors: GradDataArray<ColorInt>,
    pub(super) m_stops: GradDataArray<f32>,
    pub(super) m_count: usize,
    pub(super) m_coeffs: [f32; 3],
    pub(super) m_isOpaque: UnsafeCell<gpu::TriState>,
    pub(super) m_lastModulatedGradient: UnsafeCell<rcp<Gradient>>,
    pub(super) m_lastModulatedOpacity: UnsafeCell<f32>,
    _construction_seal: GradientConstructionSeal,
}

impl LiteRttiTypeId for Gradient {
    const LITE_RTTI_TYPE_ID: u32 = CONST_ID("Gradient");
}

impl LiteRttiCastFrom<RenderShader> for Gradient {
    unsafe fn from_base(base: *mut RenderShader) -> *mut Self {
        base.cast()
    }
}

// SAFETY: `RenderShader` is the offset-zero intrusive/RTTI base and its
// source-shaped destructor slot casts this exact complete Gradient allocation.
unsafe impl RefCntTarget for Gradient {
    fn r#ref(&self) {
        self.base.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.base.unref() };
    }
}

impl Gradient {
    fn new(
        paint_type: gpu::PaintType,
        colors: GradDataArray<ColorInt>,
        stops: GradDataArray<f32>,
        count: usize,
        coeff_x: f32,
        coeff_y: f32,
        coeff_z: f32,
    ) -> Self {
        debug_assert!(
            paint_type == gpu::PaintType::linearGradient
                || paint_type == gpu::PaintType::radialGradient
        );
        unsafe fn destroy_complete(ptr: *mut RenderShader) {
            // SAFETY: the installed source destructor slot receives the
            // offset-zero complete Gradient allocation.
            unsafe {
                drop(Box::from_raw(ptr.cast::<Gradient>()));
            }
        }
        Self {
            // SAFETY: Gradient is the complete allocation owning this base.
            base: RenderShader {
                base: RefCnt::new(),
                destroy_complete,
                m_liteTypeId: Self::LITE_RTTI_TYPE_ID,
            },
            m_paintType: paint_type,
            m_colors: colors,
            m_stops: stops,
            m_count: count,
            m_coeffs: [coeff_x, coeff_y, coeff_z],
            m_isOpaque: UnsafeCell::new(gpu::TriState::unknown),
            m_lastModulatedGradient: UnsafeCell::new(rcp::new()),
            m_lastModulatedOpacity: UnsafeCell::new(-1.0),
            _construction_seal: GradientConstructionSeal,
        }
    }

    /// Immediate intrusive construction helper. `new` is private so a
    /// complete Gradient cannot escape as a safe stack/boxed value.
    ///
    /// # Safety
    /// `count` must equal the initialized length of both paired arrays, and
    /// both arrays must remain valid source-owned values for this immediate
    /// complete-object allocation.
    pub(super) unsafe fn make_rcp(
        paint_type: gpu::PaintType,
        colors: GradDataArray<ColorInt>,
        stops: GradDataArray<f32>,
        count: usize,
        coeff_x: f32,
        coeff_y: f32,
        coeff_z: f32,
    ) -> rcp<Gradient> {
        allocate_rcp(|| Self::new(paint_type, colors, stops, count, coeff_x, coeff_y, coeff_z))
    }

    pub fn paintType(&self) -> gpu::PaintType {
        self.m_paintType
    }
    pub fn coeffs(&self) -> *const f32 {
        self.m_coeffs.as_ptr()
    }
    pub fn colors(&self) -> *const ColorInt {
        self.m_colors.get()
    }
    pub fn stops(&self) -> *const f32 {
        self.m_stops.get()
    }
    pub fn colors_slice(&self) -> &[ColorInt] {
        // SAFETY: Gradient construction validates and stores the paired
        // arrays with exactly m_count initialized elements.
        unsafe { self.m_colors.as_slice(self.m_count) }
    }
    pub fn stops_slice(&self) -> &[f32] {
        // SAFETY: Gradient construction validates and stores the paired
        // arrays with exactly m_count initialized elements.
        unsafe { self.m_stops.as_slice(self.m_count) }
    }
    pub fn count(&self) -> usize {
        self.m_count
    }
}

#[cfg(test)]
mod assertion_tests {
    use super::*;

    #[test]
    fn source_paint_type_assert_obeys_ndebug() {
        let result = std::panic::catch_unwind(|| {
            let _gradient = Gradient::new(
                gpu::PaintType::solidColor,
                GradDataArray::from_slice(&[]),
                GradDataArray::from_slice(&[]),
                0,
                0.0,
                0.0,
                0.0,
            );
        });
        match (cfg!(debug_assertions), result.is_err()) {
            (true, true) | (false, false) => {}
            (true, false) => panic!("source assert did not fire in a debug build"),
            (false, true) => panic!("source assert remained active in an NDEBUG build"),
        }
    }
}

impl Drop for Gradient {
    fn drop(&mut self) {
        // Release the single-entry cache before the source gradient arrays.
        let cached = unsafe { &mut *self.m_lastModulatedGradient.get() };
        let released = rcp::move_ctor(cached);
        drop(released);
        unsafe {
            self.m_stops.release();
            self.m_colors.release();
        }
    }
}

/// Public product-facing shader adapter. The API never exposes or constructs
/// the source `Gradient` value directly; this wrapper owns one intrusive
/// `rcp<Gradient>` and therefore keeps the complete source allocation alive.
pub struct GradientShader {
    gradient: rcp<Gradient>,
}

impl GradientShader {
    pub fn new(gradient: rcp<Gradient>) -> Option<Self> {
        (!gradient.get().is_null()).then_some(Self { gradient })
    }

    pub fn gradient(&self) -> rcp<Gradient> {
        self.gradient.clone()
    }

    pub fn source(&self) -> &Gradient {
        // SAFETY: new rejects null and this immutable shader handle owns a
        // source retain for the duration of the borrow.
        unsafe { &*self.gradient.get() }
    }

    pub fn source_base(&self) -> &RenderShader {
        &self.source().base
    }

    pub fn into_source(self) -> rcp<Gradient> {
        self.gradient
    }
}

impl nuxie_render_api::RenderShader for GradientShader {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}
