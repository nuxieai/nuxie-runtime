/*
 * Copyright 2022 Rive
 */

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/buffer_ring.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// The complete source is retained below in declaration order. The Rust
// declarations following it keep the source names, field order, defaults,
// nullable lazy owner, three-slot cursor, virtual contracts, and destructor
// boundary visible to later mechanical translations.

// /*
//  * Copyright 2022 Rive
//  */
//
// #pragma once
//
// #include "rive/renderer/gpu.hpp"
//
// namespace rive::gpu
// {
// // API-agnostic implementation of an abstract buffer ring. We use rings to
// // ensure the GPU can render one frame in parallel while the CPU prepares the
// // next frame.
// //
// // Calling mapBuffer() maps the next buffer in the ring.
// //
// // Calling unmapAndSubmitBuffer() submits the currently-mapped buffer for GPU
// // rendering, in whatever way that is meaningful for the RenderContext
// // implementation.
// //
// // This class is meant to only be used through BufferRing<>.
// class BufferRing
// {
// public:
//     BufferRing(size_t capacityInBytes) : m_capacityInBytes(capacityInBytes) {}
//     virtual ~BufferRing() {}
//
//     size_t capacityInBytes() const { return m_capacityInBytes; }
//     bool isMapped() const { return m_mapSizeInBytes != 0; }
//     size_t mapSizeInBytes() const { return m_mapSizeInBytes; }
//
//     // Maps the next buffer in the ring.
//     void* mapBuffer(size_t mapSizeInBytes)
//     {
//         assert(!isMapped());
//         assert(mapSizeInBytes > 0);
//         assert(mapSizeInBytes <= m_capacityInBytes);
//         m_submittedBufferIdx = (m_submittedBufferIdx + 1) % kBufferRingSize;
//         m_mapSizeInBytes = mapSizeInBytes;
//         return onMapBuffer(m_submittedBufferIdx, m_mapSizeInBytes);
//     }
//
//     // Submits the currently-mapped buffer for GPU rendering, in whatever way
//     // that is meaningful for the RenderContext implementation.
//     void unmapAndSubmitBuffer()
//     {
//         assert(isMapped());
//         onUnmapAndSubmitBuffer(m_submittedBufferIdx, m_mapSizeInBytes);
//         m_mapSizeInBytes = 0;
//     }
//
// protected:
//     int submittedBufferIdx() const
//     {
//         assert(!isMapped());
//         return m_submittedBufferIdx;
//     }
//
//     virtual void* onMapBuffer(int bufferIdx, size_t mapSizeInBytes) = 0;
//     virtual void onUnmapAndSubmitBuffer(int bufferIdx,
//                                         size_t mapSizeInBytes) = 0;
//
//     uint8_t* shadowBuffer() const
//     {
//         if (m_shadowBuffer == nullptr && m_capacityInBytes > 0)
//         {
//             m_shadowBuffer.reset(new uint8_t[m_capacityInBytes]);
//         }
//         return m_shadowBuffer.get();
//     }
//
// private:
//     size_t m_capacityInBytes;
//     size_t m_mapSizeInBytes = 0;
//     int m_submittedBufferIdx = 0;
//
//     // Lazy-allocated CPU buffer for when buffer mapping isn't supported by the
//     // API.
//     mutable std::unique_ptr<uint8_t[]> m_shadowBuffer;
// };
//
// // BufferRing that resides solely in CPU memory, and therefore doesn't require a
// // ring.
// class HeapBufferRing : public BufferRing
// {
// public:
//     HeapBufferRing(size_t capacityInBytes) : BufferRing(capacityInBytes) {}
//
//     uint8_t* contents() const { return shadowBuffer(); }
//
// protected:
//     void* onMapBuffer(int bufferIdx, size_t mapSizeInBytes) override
//     {
//         return shadowBuffer();
//     }
//     void onUnmapAndSubmitBuffer(int bufferIdx, size_t mapSizeInBytes) override
//     {}
// };
// } // namespace rive::gpu

// Rust declaration pass for the complete source header above. The source
// comments are intentionally retained verbatim; this file is a mechanical
// owner and is not the place to introduce a cross-backend GPU abstraction.
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::cell::RefCell;
use core::ffi::c_void;

// Mapped source dependency: renderer/include/rive/renderer/gpu.hpp. The
// include owns this fixed three-slot constant in the pinned declaration.
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::kBufferRingSize;

// class BufferRing
//
// The C++ class has a protected pure-virtual seam. Rust spells that seam as a
// contract, while this value carries the complete base-class state and its
// public state-machine methods.
#[repr(C)]
pub struct BufferRing {
    // size_t m_capacityInBytes;
    m_capacity_in_bytes: usize,
    // size_t m_mapSizeInBytes = 0;
    m_map_size_in_bytes: usize,
    // int m_submittedBufferIdx = 0;
    m_submitted_buffer_idx: i32,
    // mutable std::unique_ptr<uint8_t[]> m_shadowBuffer;
    //
    // RefCell supplies only the source `mutable` interior-mutability needed by
    // const shadowBuffer()/contents() calls. The owned value remains exactly
    // the source's nullable Option<Box<[u8]>> lazy CPU allocation.
    m_shadow_buffer: RefCell<Option<Box<[u8]>>>,
}

// protected virtual void* onMapBuffer(int bufferIdx, size_t mapSizeInBytes) = 0;
// protected virtual void onUnmapAndSubmitBuffer(int bufferIdx,
//                                               size_t mapSizeInBytes) = 0;
pub trait BufferRingContract {
    fn bufferRing(&self) -> &BufferRing;
    fn bufferRingMut(&mut self) -> &mut BufferRing;
    fn onMapBuffer(&mut self, bufferIdx: i32, mapSizeInBytes: usize) -> *mut c_void;
    fn onUnmapAndSubmitBuffer(&mut self, bufferIdx: i32, mapSizeInBytes: usize);

    /// Backend source owners expose the submitted native buffer only at the
    /// concrete Metal seam; generic heap rings intentionally return none.
    fn submittedHandle(&self) -> Option<crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::Handle> {
        None
    }

    fn mapSizeInBytes(&self) -> usize {
        self.bufferRing().mapSizeInBytes()
    }

    // C++ public non-virtual wrapper. Dispatch stays on `self`, so callers use
    // the authored one-argument form even through `Box<dyn BufferRingContract>`.
    fn mapBuffer(&mut self, mapSizeInBytes: usize) -> *mut c_void {
        let (bufferIdx, mapSizeInBytes) = self.bufferRingMut().beginMap(mapSizeInBytes);
        self.onMapBuffer(bufferIdx, mapSizeInBytes)
    }

    fn unmapAndSubmitBuffer(&mut self) {
        let (bufferIdx, mapSizeInBytes) = self.bufferRing().submittedMap();
        self.onUnmapAndSubmitBuffer(bufferIdx, mapSizeInBytes);
        self.bufferRingMut().finishUnmap();
    }
}

impl BufferRing {
    // BufferRing(size_t capacityInBytes) : m_capacityInBytes(capacityInBytes) {}
    pub fn new(capacityInBytes: usize) -> Self {
        Self {
            m_capacity_in_bytes: capacityInBytes,
            m_map_size_in_bytes: 0,
            m_submitted_buffer_idx: 0,
            m_shadow_buffer: RefCell::new(None),
        }
    }

    // virtual ~BufferRing() {}
    // The C++ virtual destructor is an explicit Rust Drop owner boundary; the
    // boxed lazy shadow allocation is released by the field owner.

    // size_t capacityInBytes() const { return m_capacityInBytes; }
    pub fn capacityInBytes(&self) -> usize {
        self.m_capacity_in_bytes
    }

    // bool isMapped() const { return m_mapSizeInBytes != 0; }
    pub fn isMapped(&self) -> bool {
        self.m_map_size_in_bytes != 0
    }

    // size_t mapSizeInBytes() const { return m_mapSizeInBytes; }
    pub fn mapSizeInBytes(&self) -> usize {
        self.m_map_size_in_bytes
    }

    // void* mapBuffer(size_t mapSizeInBytes)
    //
    // `hooks` is the explicit Rust spelling of C++ pure-virtual dispatch on
    // the concrete BufferRing subclass. It does not alter assertion order,
    // cursor rotation, map publication, or callback arguments.
    fn beginMap(&mut self, mapSizeInBytes: usize) -> (i32, usize) {
        // assert(!isMapped());
        debug_assert!(!self.isMapped());
        // assert(mapSizeInBytes > 0);
        debug_assert!(mapSizeInBytes > 0);
        // assert(mapSizeInBytes <= m_capacityInBytes);
        debug_assert!(mapSizeInBytes <= self.m_capacity_in_bytes);
        // m_submittedBufferIdx = (m_submittedBufferIdx + 1) % kBufferRingSize;
        self.m_submitted_buffer_idx = (self.m_submitted_buffer_idx + 1) % kBufferRingSize;
        // m_mapSizeInBytes = mapSizeInBytes;
        self.m_map_size_in_bytes = mapSizeInBytes;
        // return onMapBuffer(m_submittedBufferIdx, m_mapSizeInBytes);
        (self.m_submitted_buffer_idx, self.m_map_size_in_bytes)
    }

    // void unmapAndSubmitBuffer()
    fn submittedMap(&self) -> (i32, usize) {
        // assert(isMapped());
        debug_assert!(self.isMapped());
        (self.m_submitted_buffer_idx, self.m_map_size_in_bytes)
    }

    fn finishUnmap(&mut self) {
        // m_mapSizeInBytes = 0;
        self.m_map_size_in_bytes = 0;
    }

    // int submittedBufferIdx() const
    pub(crate) fn submittedBufferIdx(&self) -> i32 {
        // assert(!isMapped());
        debug_assert!(!self.isMapped());
        self.m_submitted_buffer_idx
    }

    // uint8_t* shadowBuffer() const
    pub(crate) fn shadowBuffer(&self) -> *mut u8 {
        let mut shadow_buffer = self.m_shadow_buffer.borrow_mut();
        // if (m_shadowBuffer == nullptr && m_capacityInBytes > 0)
        if shadow_buffer.is_none() && self.m_capacity_in_bytes > 0 {
            // m_shadowBuffer.reset(new uint8_t[m_capacityInBytes]);
            *shadow_buffer = Some(vec![0u8; self.m_capacity_in_bytes].into_boxed_slice());
        }
        // return m_shadowBuffer.get();
        shadow_buffer
            .as_deref_mut()
            .map_or(core::ptr::null_mut(), |bytes| bytes.as_mut_ptr())
    }
}

impl Drop for BufferRing {
    fn drop(&mut self) {}
}

// class HeapBufferRing : public BufferRing
//
// Rust has no C++ base subobject or virtual-inheritance layout. The embedded
// `base` field is the owned BufferRing base and preserves its destruction and
// state ownership boundary; BufferRingContract is the override seam.
#[repr(C)]
pub struct HeapBufferRing {
    pub(crate) base: BufferRing,
}

impl HeapBufferRing {
    // HeapBufferRing(size_t capacityInBytes) : BufferRing(capacityInBytes) {}
    pub fn new(capacityInBytes: usize) -> Self {
        Self {
            base: BufferRing::new(capacityInBytes),
        }
    }

    // uint8_t* contents() const { return shadowBuffer(); }
    pub fn contents(&self) -> *mut u8 {
        self.base.shadowBuffer()
    }
}

impl BufferRingContract for HeapBufferRing {
    fn bufferRing(&self) -> &BufferRing {
        &self.base
    }

    fn bufferRingMut(&mut self) -> &mut BufferRing {
        &mut self.base
    }

    // void* onMapBuffer(int bufferIdx, size_t mapSizeInBytes) override
    fn onMapBuffer(&mut self, bufferIdx: i32, mapSizeInBytes: usize) -> *mut c_void {
        // The source names both arguments in this override but does not read
        // them; preserving that no-op is part of the complete declaration.
        let _ = (bufferIdx, mapSizeInBytes);
        // return shadowBuffer();
        self.base.shadowBuffer().cast::<c_void>()
    }

    // void onUnmapAndSubmitBuffer(int bufferIdx, size_t mapSizeInBytes) override
    // {}
    fn onUnmapAndSubmitBuffer(&mut self, bufferIdx: i32, mapSizeInBytes: usize) {
        let _ = (bufferIdx, mapSizeInBytes);
    }
}

// The derived class has no additional destructor body; dropping it first drops
// its owned BufferRing base, which releases the lazy shadow allocation exactly
// at the C++ virtual-destruction boundary.
impl Drop for HeapBufferRing {
    fn drop(&mut self) {}
}
