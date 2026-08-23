/*
 * Copyright 2022 Rive
 */

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/rive_render_buffer.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// The complete source is retained below in declaration order. The Rust
// declaration following it keeps the RenderBuffer base owner, inherited map
// and unmap APIs, ring state, defaults, and destructor boundary visible to
// later mechanical translations.

// /*
//  * Copyright 2022 Rive
//  */
//
// #pragma once
//
// #include "rive/renderer.hpp"
// #include "rive/renderer/gpu.hpp"
//
// namespace rive
// {
// // RenderBuffer with additional indices to track the "front" and "back" buffers,
// // assuming a ring of gpu::kBufferRingSize buffers.
// class RiveRenderBuffer : public RenderBuffer
// {
// protected:
//     RiveRenderBuffer(RenderBufferType type,
//                      RenderBufferFlags flags,
//                      size_t sizeInBytes) :
//         RenderBuffer(type, flags, sizeInBytes)
//     {}
//
//     // Returns the index of the buffer to map and update, prior to rendering.
//     int backBufferIdx() const { return m_backBufferIdx; }
//
//     // Returns the index of the buffer to submit with rendering commands.
//     // Automatically advances the buffer ring if the RenderBuffer is dirty.
//     int frontBufferIdx()
//     {
//         if (checkAndResetDirty())
//         {
//             // The update buffer is dirty. Advance the buffer ring.
//             m_frontBufferIdx = m_backBufferIdx;
//             m_backBufferIdx = (m_backBufferIdx + 1) % gpu::kBufferRingSize;
//         }
//         return m_frontBufferIdx;
//     }
//
// private:
//     int m_backBufferIdx = 0;
//     int m_frontBufferIdx = -1;
// };
// } // namespace rive

// Rust declaration pass for the complete source header above. The source
// comments are intentionally retained verbatim; this file is a mechanical
// owner and is not the place to introduce a cross-backend GPU abstraction.
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::c_void;
use std::any::Any;
use std::rc::Rc;

// Mapped source dependency: include/rive/renderer.hpp. The RenderBuffer
// owner supplies the inherited map/unmap APIs and dirty-state transition.
use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderBufferContract, RenderBufferFlags, RenderBufferType,
};
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::LiteRttiTypeId;

// Mapped source dependency: renderer/include/rive/renderer/gpu.hpp. The
// include owns this fixed three-slot constant in the pinned declaration.
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::kBufferRingSize;

// class RiveRenderBuffer : public RenderBuffer
//
// Rust has no implicit C++ base-class method or subobject inheritance. The
// `base` field is the complete RenderBuffer owner and is declared first to
// preserve the source base topology. The two ring indices follow it in source
// declaration order; dropping this owner therefore drops front, back, then
// the base, matching C++ derived-before-base destruction.
#[repr(C)]
pub struct RiveRenderBuffer {
    // public RenderBuffer base class
    pub(crate) base: RenderBuffer,

    // int m_backBufferIdx = 0;
    m_backBufferIdx: i32,
    // int m_frontBufferIdx = -1;
    m_frontBufferIdx: i32,
}

impl RiveRenderBuffer {
    // RiveRenderBuffer(RenderBufferType type,
    //                  RenderBufferFlags flags,
    //                  size_t sizeInBytes) :
    //     RenderBuffer(type, flags, sizeInBytes)
    // {}
    //
    // The source constructor is protected. `pub(crate)` keeps construction
    // available to the translated backend owner while withholding it from the
    // public crate API.
    /// # Safety
    /// `Owner` must contain this RiveRenderBuffer, and therefore its nested
    /// RenderBuffer base, at offset zero for the complete allocation lifetime.
    pub(crate) unsafe fn new_for_owner<Owner: RenderBufferContract + LiteRttiTypeId>(
        type_: RenderBufferType,
        flags: RenderBufferFlags,
        sizeInBytes: usize,
    ) -> Self {
        let base = unsafe { RenderBuffer::new_for_owner::<Owner>(type_, flags, sizeInBytes) };
        Self {
            base,
            m_backBufferIdx: 0,
            m_frontBufferIdx: -1,
        }
    }

    // RenderBuffer's inherited public API remains callable through the
    // derived owner in C++. These forwarding methods preserve that mapping
    // surface without copying or reimplementing base state.
    // void* map();
    pub fn map(&mut self) -> *mut c_void {
        self.base.map()
    }

    // void unmap();
    pub fn unmap(&mut self) {
        self.base.unmap();
    }

    // RenderBufferType type() const { return m_type; }
    pub fn r#type(&self) -> RenderBufferType {
        self.base.r#type()
    }

    // RenderBufferFlags flags() const { return m_flags; }
    pub fn flags(&self) -> RenderBufferFlags {
        self.base.flags()
    }

    // size_t sizeInBytes() const { return m_sizeInBytes; }
    pub fn sizeInBytes(&self) -> usize {
        self.base.sizeInBytes()
    }

    // Returns the index of the buffer to map and update, prior to rendering.
    // int backBufferIdx() const { return m_backBufferIdx; }
    pub(crate) fn backBufferIdx(&self) -> i32 {
        self.m_backBufferIdx
    }

    // Returns the index of the buffer to submit with rendering commands.
    // Automatically advances the buffer ring if the RenderBuffer is dirty.
    // int frontBufferIdx()
    pub(crate) fn frontBufferIdx(&mut self) -> i32 {
        // if (checkAndResetDirty())
        if self.base.checkAndResetDirty() {
            // The update buffer is dirty. Advance the buffer ring.
            // m_frontBufferIdx = m_backBufferIdx;
            self.m_frontBufferIdx = self.m_backBufferIdx;
            // m_backBufferIdx = (m_backBufferIdx + 1) % gpu::kBufferRingSize;
            self.m_backBufferIdx = (self.m_backBufferIdx + 1) % kBufferRingSize;
        }
        // return m_frontBufferIdx;
        self.m_frontBufferIdx
    }
}

impl nuxie_render_api::RenderBuffer for RiveRenderBuffer {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
    fn buffer_type(&self) -> nuxie_render_api::RenderBufferType {
        match self.r#type() {
            RenderBufferType::index => nuxie_render_api::RenderBufferType::Index,
            RenderBufferType::vertex => nuxie_render_api::RenderBufferType::Vertex,
        }
    }
    fn flags(&self) -> nuxie_render_api::RenderBufferFlags {
        match self.flags() {
            RenderBufferFlags::none => nuxie_render_api::RenderBufferFlags::None,
            RenderBufferFlags::mappedOnceAtInitialization => {
                nuxie_render_api::RenderBufferFlags::MappedOnceAtInitialization
            }
        }
    }
    fn size_in_bytes(&self) -> usize {
        self.sizeInBytes()
    }
    fn map_mut(&mut self) -> &mut [u8] {
        let size = self.sizeInBytes();
        let ptr = self.map().cast::<u8>();
        let ptr = if size == 0 && ptr.is_null() {
            core::ptr::NonNull::<u8>::dangling().as_ptr()
        } else {
            ptr
        };
        assert!(
            !ptr.is_null(),
            "source RenderBuffer::map returned null for a nonempty buffer"
        );
        // SAFETY: the source map contract returns a writable span of exactly
        // sizeInBytes bytes for the live buffer mapping.
        unsafe { core::slice::from_raw_parts_mut(ptr, size) }
    }
    fn unmap(&mut self) {
        self.unmap();
    }
}

/// Product-facing owner for the backend's exact intrusive RenderBuffer
/// allocation. The backend may append its own complete-owner fields after the
/// offset-zero RiveRenderBuffer/RenderBuffer chain, so this wrapper retains the
/// source base pointer and dispatches through its installed virtual slots.
#[derive(Clone)]
pub(crate) struct RenderResourceDomain {
    identity: Rc<()>,
}

impl RenderResourceDomain {
    /// Creates one opaque execution-domain identity. Clones compare as the
    /// same domain because they retain the same allocation identity.
    pub(crate) fn new() -> Self {
        Self {
            identity: Rc::new(()),
        }
    }

    pub(crate) fn same_domain(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.identity, &other.identity)
    }
}

#[derive(Clone)]
struct AttachedExecutionDomain {
    resource_domain: RenderResourceDomain,
    // Retains the actual backend execution owner until after source teardown.
    _domain_guard: Rc<dyn Any>,
}

pub struct RiveRenderBufferHandle {
    source: rcp<RenderBuffer>,
    // Declared after source so the intrusive source allocation is released
    // before its backend execution owner. Identity and lifetime are bundled so
    // a handle can never carry only one half of its execution-domain edge.
    execution_domain: Option<AttachedExecutionDomain>,
}

impl RiveRenderBufferHandle {
    /// # Safety
    /// `source` must be a fresh RenderContext factory result with no second
    /// safe product-wrapper mutation authority for the same allocation.
    pub(crate) unsafe fn from_source(source: rcp<RenderBuffer>) -> Option<Self> {
        (!source.get().is_null()).then_some(Self {
            source,
            execution_domain: None,
        })
    }

    /// Attach the opaque identity and owner of this buffer's execution domain
    /// together. Consuming self prevents a transient duplicate mutable handle
    /// while installing the single lifetime/identity edge.
    pub(crate) fn with_execution_domain(
        mut self,
        resource_domain: RenderResourceDomain,
        domain_guard: Rc<dyn Any>,
    ) -> Self {
        assert!(
            self.execution_domain.is_none(),
            "buffer execution domain already attached"
        );
        self.execution_domain = Some(AttachedExecutionDomain {
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

    fn source(&self) -> &RenderBuffer {
        // SAFETY: construction rejects null and this handle retains the
        // complete backend allocation through its source base.
        unsafe { &*self.source.get() }
    }

    fn source_mut(&mut self) -> &mut RenderBuffer {
        // SAFETY: the handle is non-Clone and does not expose its owning rcp.
        unsafe { &mut *self.source.get() }
    }

    /// Retains the exact backend buffer for a source Renderer call after the
    /// caller has validated `belongs_to` against the active frame's domain.
    ///
    /// # Safety
    /// The returned owner must remain scoped beneath the matching execution-
    /// domain guard and must not escape the validated native frame operation.
    pub(crate) unsafe fn source_owner_unchecked(&self) -> rcp<RenderBuffer> {
        self.source.clone()
    }
}

impl nuxie_render_api::RenderBuffer for RiveRenderBufferHandle {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn buffer_type(&self) -> nuxie_render_api::RenderBufferType {
        match self.source().r#type() {
            RenderBufferType::index => nuxie_render_api::RenderBufferType::Index,
            RenderBufferType::vertex => nuxie_render_api::RenderBufferType::Vertex,
        }
    }

    fn flags(&self) -> nuxie_render_api::RenderBufferFlags {
        match self.source().flags() {
            RenderBufferFlags::none => nuxie_render_api::RenderBufferFlags::None,
            RenderBufferFlags::mappedOnceAtInitialization => {
                nuxie_render_api::RenderBufferFlags::MappedOnceAtInitialization
            }
        }
    }

    fn size_in_bytes(&self) -> usize {
        self.source().sizeInBytes()
    }

    fn map_mut(&mut self) -> &mut [u8] {
        let size = self.source().sizeInBytes();
        let mapped = self.source_mut().map().cast::<u8>();
        let mapped = if size == 0 && mapped.is_null() {
            core::ptr::NonNull::<u8>::dangling().as_ptr()
        } else {
            mapped
        };
        assert!(
            !mapped.is_null(),
            "source RenderBuffer::map returned null for a nonempty buffer"
        );
        // SAFETY: the source map virtual returns a writable range of exactly
        // sizeInBytes bytes. Rust still requires a nonnull pointer for len 0.
        unsafe { core::slice::from_raw_parts_mut(mapped, size) }
    }

    fn unmap(&mut self) {
        self.source_mut().unmap();
    }
}

// The source has no explicit RiveRenderBuffer destructor body; its inherited
// RenderBuffer destructor is virtual. The base's zero-release dispatch points
// at the complete owner so derived destruction precedes base destruction.
impl Drop for RiveRenderBuffer {
    fn drop(&mut self) {}
}
