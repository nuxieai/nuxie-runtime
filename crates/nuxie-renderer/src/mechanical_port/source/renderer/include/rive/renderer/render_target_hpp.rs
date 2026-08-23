/*
 * Copyright 2023 Rive
 */

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/render_target.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// #pragma once

// #include "rive/refcnt.hpp"
//
// #include "rive/math/aabb.hpp"
// #include "rive/math/simd.hpp"

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use crate::mechanical_port::source::include::rive::refcnt_hpp::{RefCnt, RefCntTarget};

// `IAABB` is the `TAABB<int32_t>` value returned by the pinned source.  The
// source-shaped declaration is kept local to this header translation because
// the upstream `rive/math/aabb.hpp` owner is not otherwise part of this
// mechanical source set.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IAABB {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

// `uint2` is `rive::simd::uvec<2>` in the pinned SIMD header.  A fixed-width
// array retains its two uint32 lanes and aggregate return shape without
// introducing a backend SIMD dependency into this source-shaped owner.
pub type uint2 = [u32; 2];

// namespace rive::gpu
// {
// // Wraps a backend-specific buffer that RenderContext draws into.
// class RenderTarget : public RefCnt<RenderTarget>
// {
// public:
//     virtual ~RenderTarget() {}
//
//     uint32_t width() const { return m_width; }
//     uint32_t height() const { return m_height; }
//     uint2 size() const { return {m_width, m_height}; }
//     IAABB bounds() const
//     {
//         return IAABB{0,
//                      0,
//                      static_cast<int>(m_width),
//                      static_cast<int>(m_height)};
//     }
//
// protected:
//     RenderTarget(uint32_t width, uint32_t height) :
//         m_width(width), m_height(height)
//     {}
//
// private:
//     uint32_t m_width;
//     uint32_t m_height;
// };
// } // namespace rive::gpu

// Rust has no C++ derived-class base subobject. The first field is the
// intrusive `RefCnt<RenderTarget>` base, preserving the source base topology;
// `m_width` and `m_height` remain private, immutable-after-construction
// payload fields in their exact source order. `RefCntTarget` supplies the
// source's static-cast zero-reference deletion hook through this owner.
#[repr(C)]
pub struct RenderTarget {
    // public RefCnt<RenderTarget> base class
    pub(crate) base: RefCnt<RenderTarget>,
    pub(crate) destroy_complete: unsafe fn(*mut RenderTarget),

    // uint32_t m_width;
    m_width: u32,
    // uint32_t m_height;
    m_height: u32,
}

// SAFETY: `base` is the offset-zero field and `destroy_complete` deletes the
// complete derived render-target owner selected at construction.
unsafe impl RefCntTarget for RenderTarget {
    // RefCnt's inherited `ref()` operation.
    fn r#ref(&self) {
        self.base.r#ref();
    }

    // RefCnt's inherited `unref()` operation.
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }

    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { ((*ptr).destroy_complete)(ptr.cast_mut()) };
    }
}

impl RenderTarget {
    // virtual ~RenderTarget() {}
    // The empty virtual destructor is represented by Rust's default drop glue;
    // the intrusive base remains the owner of reference-counted deletion.

    // uint32_t width() const { return m_width; }
    pub fn width(&self) -> u32 {
        self.m_width
    }

    // uint32_t height() const { return m_height; }
    pub fn height(&self) -> u32 {
        self.m_height
    }

    // uint2 size() const { return {m_width, m_height}; }
    pub fn size(&self) -> uint2 {
        [self.m_width, self.m_height]
    }

    // IAABB bounds() const
    // {
    //     return IAABB{0,
    //                  0,
    //                  static_cast<int>(m_width),
    //                  static_cast<int>(m_height)};
    // }
    pub fn bounds(&self) -> IAABB {
        IAABB {
            left: 0,
            top: 0,
            right: self.m_width as i32,
            bottom: self.m_height as i32,
        }
    }

    // protected:
    // RenderTarget(uint32_t width, uint32_t height) :
    //     m_width(width), m_height(height)
    // {}
    //
    // The source constructor is protected. `pub(crate)` keeps construction
    // available to the translated backend owner while withholding it from the
    // public crate API.
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self {
            base: RefCnt::new(),
            destroy_complete: |ptr| unsafe { drop(Box::from_raw(ptr)) },
            m_width: width,
            m_height: height,
        }
    }
}
