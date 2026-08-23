/*
 * Copyright 2025 Rive
 */

// #pragma once

// #include "rive/renderer/gpu_resource.hpp"
// #include "utils/lite_rtti.hpp"
// #include "rive/renderer/ore/ore_types.hpp"

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/ore/ore_buffer.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::ore_types_hpp::BufferUsage;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

use super::super::gpu_resource_hpp::{GPUResource, GpuResourcePayload};

// namespace rive::ore

pub trait BufferApi {
    fn size(&self) -> u32;
    fn usage(&self) -> BufferUsage;
    /// Updates exactly `size` bytes from the borrowed span.
    fn update(&self, data: &[u8], size: u32, offset: u32) -> Result<(), BufferUpdateError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferUpdateError {
    WrongResourceKind,
    WrongExecutionDomain,
    SourceTooShort,
    RangeOverflow,
    RangeOutOfBounds,
}

// class Context;
// The source forward declaration is retained for the friend relationship
// below. Context is owned by its own translation unit.

// class Buffer : public rive::gpu::GPUResource,
//                public ENABLE_LITE_RTTI(Buffer)
//
#[repr(C)]
pub struct BufferMembers {
    pub(crate) m_size: u32,
    pub(crate) m_usage: BufferUsage,
}

#[repr(C)]
pub struct Buffer {
    pub(crate) base: ManuallyDrop<GPUResource>,
    pub(crate) members: ManuallyDrop<BufferMembers>,
}

impl Deref for Buffer {
    type Target = BufferMembers;

    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.members
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        #[cfg(test)]
        super::super::gpu_resource_hpp::record_resource_drop_stage("Buffer.base");
        unsafe { ManuallyDrop::drop(&mut self.base) };
    }
}

unsafe impl GpuResourcePayload for Buffer {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base
    }
}

impl Buffer {
    // public:
    // uint32_t size() const { return m_size; }
    pub fn size(&self) -> u32 {
        self.m_size
    }

    // BufferUsage usage() const { return m_usage; }
    pub fn usage(&self) -> BufferUsage {
        self.m_usage
    }

    // virtual void update(const void* data,
    //                     uint32_t size,
    //                     uint32_t offset = 0) = 0;
    //
    // `BufferApi` above is the callable Rust virtual-dispatch surface. Its
    // borrowed span preserves provenance while `size` remains independent.

    // virtual ~Buffer() = default;
    // Rust's default drop glue supplies the virtual-destructor boundary for
    // the concrete resource owner; no extra state is introduced here.

    // protected:
    // friend class Context;
    // friend class RenderPass;

    // Buffer(uint32_t size, BufferUsage usage) :
    //     rive::gpu::GPUResource(nullptr), m_size(size), m_usage(usage)
    // {}
    pub(crate) fn new(size: u32, usage: BufferUsage) -> Self {
        Self {
            base: ManuallyDrop::new(GPUResource::new(None)),
            members: ManuallyDrop::new(BufferMembers {
                m_size: size,
                m_usage: usage,
            }),
        }
    }

    // Buffer(rcp<rive::gpu::GPUResourceManager> manager,
    //        uint32_t size,
    //        BufferUsage usage) :
    //     rive::gpu::GPUResource(std::move(manager)), m_size(size), m_usage(usage)
    // {}
    // The manager-taking form is represented when the complete concrete
    // payload is published with `ResourceHandle::new(Some(manager), payload)`;
    // duplicating it here would create a second base owner.
}

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_base_preserves_size_and_usage() {
        let buffer = Buffer::new(4096, BufferUsage::uniform);
        assert_eq!(buffer.size(), 4096);
        assert_eq!(buffer.usage(), BufferUsage::uniform);
    }
}
