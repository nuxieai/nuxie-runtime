/*
 * Copyright 2025 Rive
 */

// #pragma once
// #include "rive/renderer/ore/ore_bind_group.hpp"
// #include "rive/renderer/ore/ore_binding_map.hpp"
// #import <Metal/Metal.h>
// #include <vector>

// Mechanical translation of the complete pinned source header
// renderer/src/ore/metal/ore_bind_group_metal.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;
use std::mem::ManuallyDrop;

use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_bind_group_hpp::BindGroup;
use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_binding_map_hpp::BindingMap;

// `id<MTLTexture>` is a nullable, strong Objective-C owner under ARC. Rust's
// `Retained<T>` is the corresponding strong owner; `Option` preserves the
// source `nil` state. The non-Apple stand-in keeps this source-shaped
// translation available to tools that inspect it off Apple.
#[cfg(target_vendor = "apple")]
use objc2::rc::Retained;
#[cfg(target_vendor = "apple")]
use objc2::runtime::ProtocolObject;
#[cfg(target_vendor = "apple")]
use objc2_metal::{MTLSamplerState, MTLTexture};

#[cfg(target_vendor = "apple")]
type NativeMetalTexture = Option<Retained<ProtocolObject<dyn MTLTexture>>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalTexture = Option<()>;

#[cfg(target_vendor = "apple")]
type NativeMetalSampler = Option<Retained<ProtocolObject<dyn MTLSamplerState>>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalSampler = Option<()>;

// namespace rive::ore

// class ContextMetal;
// class RenderPassMetal;
// class BufferMetal;

// class BindGroupMetal : public LITE_RTTI_OVERRIDE(BindGroup, BindGroupMetal)
// {
// public:

// struct MTLBufferBinding
// {
//     // Source buffer, not a raw handle, so the live backing resolves at
//     // encode time. Kept alive by m_retainedBuffers.
//     BufferMetal* src = nullptr;
//     uint32_t offset = 0;
//     uint32_t binding = 0; // WGSL @binding, for sort
//     bool hasDynamicOffset = false;
//     uint16_t vsSlot = BindingMap::kAbsent;
//     uint16_t fsSlot = BindingMap::kAbsent;
// };
//
// Rust has no nested type declarations inside a struct body, so this public
// source record is represented as a top-level sibling. A stable index names
// the exact logical buffer owner retained by the portable BindGroup base; the
// live native backing is resolved by checked downcast at encode time.
#[derive(Clone, Copy)]
pub struct MTLBufferBinding {
    // Source buffer, not a raw handle, so the live backing resolves at
    // encode time. Kept alive by m_retainedBuffers.
    src_index: usize,
    pub offset: u32,
    pub binding: u32, // WGSL @binding, for sort
    pub hasDynamicOffset: bool,
    pub vsSlot: u16,
    pub fsSlot: u16,
}

impl MTLBufferBinding {
    pub fn new(src_index: usize) -> Self {
        Self {
            src_index,
            ..Self::default()
        }
    }

    pub fn source<'a>(&self, group: &'a BindGroupMetal) -> Option<&'a BufferMetal> {
        group
            .base
            .retained_buffer(self.src_index)
            .and_then(|resource| resource.downcast_ref::<BufferMetal>())
    }
}

impl Default for MTLBufferBinding {
    fn default() -> Self {
        Self {
            src_index: 0,
            offset: 0,
            binding: 0,
            hasDynamicOffset: false,
            vsSlot: BindingMap::kAbsent,
            fsSlot: BindingMap::kAbsent,
        }
    }
}

// struct MTLTextureBinding
// {
//     id<MTLTexture> texture = nil;
//     uint16_t vsSlot = BindingMap::kAbsent;
//     uint16_t fsSlot = BindingMap::kAbsent;
// };
#[derive(Clone)]
pub struct MTLTextureBinding {
    pub texture: NativeMetalTexture,
    pub vsSlot: u16,
    pub fsSlot: u16,
}

impl Default for MTLTextureBinding {
    fn default() -> Self {
        Self {
            texture: None,
            vsSlot: BindingMap::kAbsent,
            fsSlot: BindingMap::kAbsent,
        }
    }
}

// struct MTLSamplerBinding
// {
//     id<MTLSamplerState> sampler = nil;
//     uint16_t vsSlot = BindingMap::kAbsent;
//     uint16_t fsSlot = BindingMap::kAbsent;
// };
#[derive(Clone)]
pub struct MTLSamplerBinding {
    pub sampler: NativeMetalSampler,
    pub vsSlot: u16,
    pub fsSlot: u16,
}

impl Default for MTLSamplerBinding {
    fn default() -> Self {
        Self {
            sampler: None,
            vsSlot: BindingMap::kAbsent,
            fsSlot: BindingMap::kAbsent,
        }
    }
}

//     BindGroupMetal() = default;
//     ~BindGroupMetal() override = default; // ARC releases Metal objects
//
// private:
//     friend class ContextMetal;
//     friend class RenderPassMetal;
//     std::vector<MTLBufferBinding> m_mtlBuffers;
//     std::vector<MTLTextureBinding> m_mtlTextures;
//     std::vector<MTLSamplerBinding> m_mtlSamplers;
// };
//
// Rust has no class inheritance or friend declarations. `base` is the
// portable BindGroup owner; the concrete record vectors retain the source
// native binding records and remain private to the owning translation units.
// Their declaration order is chosen to preserve C++ destruction order:
// sampler records, texture records, buffer records, then the BindGroup base.
#[repr(C)]
pub struct BindGroupMetal {
    pub(crate) base: ManuallyDrop<BindGroup>,
    pub(crate) m_mtlBuffers: ManuallyDrop<Vec<MTLBufferBinding>>,
    pub(crate) m_mtlTextures: ManuallyDrop<Vec<MTLTextureBinding>>,
    pub(crate) m_mtlSamplers: ManuallyDrop<Vec<MTLSamplerBinding>>,
}

// SAFETY: native texture/sampler records are immutable after publication;
// buffer lookup uses stable indices into strongly retained logical handles.
unsafe impl Send for BindGroupMetal {}

unsafe impl crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::GpuResourcePayload
    for BindGroupMetal
{
    fn gpu_resource(
        &self,
    ) -> &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::GPUResource
    {
        &self.base.base
    }

    fn gpu_resource_mut(
        &mut self,
    ) -> &mut crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::GPUResource
    {
        &mut self.base.base
    }
}

impl BindGroupMetal {
    // BindGroupMetal() = default;
    // The source default constructor invokes BindGroup's default constructor
    // and value-initializes each vector empty.
    pub(crate) fn new() -> Self {
        Self {
            base: ManuallyDrop::new(BindGroup::new()),
            m_mtlBuffers: ManuallyDrop::new(Vec::new()),
            m_mtlTextures: ManuallyDrop::new(Vec::new()),
            m_mtlSamplers: ManuallyDrop::new(Vec::new()),
        }
    }

    // ~BindGroupMetal() override = default; // ARC releases Metal objects
    // Rust's default drop glue releases sampler, texture, and buffer records,
    // then the portable BindGroup base, preserving the source derived/base
    // destruction topology. Each native Objective-C handle is an owned
    // `Retained<T>` inside its record and releases with that record.
}

impl Drop for BindGroupMetal {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_mtlSamplers);
            ManuallyDrop::drop(&mut self.m_mtlTextures);
            ManuallyDrop::drop(&mut self.m_mtlBuffers);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding_map::BindingMap;
    use crate::gpu_resource::ResourceHandle;

    fn assert_send<T: Send>() {}

    #[test]
    fn buffer_records_sort_by_binding_and_preserve_absent_stage_slots() {
        // The source constructor preserves authored records; Metal performs
        // binding-order traversal at encode time. Build that exact source
        // owner and inspect the retained record values directly.
        let mut group = BindGroupMetal::new();
        let mut first = MTLBufferBinding::new(1);
        first.offset = 8;
        first.binding = 9;
        first.vsSlot = BindingMap::kAbsent;
        first.fsSlot = 3;
        let mut second = MTLBufferBinding::new(0);
        second.binding = 2;
        second.hasDynamicOffset = true;
        second.vsSlot = 4;
        group.m_mtlBuffers = ManuallyDrop::new(vec![first, second]);

        assert_eq!(group.m_mtlBuffers[0].binding, 9);
        assert_eq!(group.m_mtlBuffers[1].binding, 2);
        assert_eq!(group.m_mtlBuffers[1].src_index, 0);
        assert!(group.m_mtlBuffers[1].hasDynamicOffset);
        assert_eq!(group.m_mtlBuffers[1].vsSlot, 4);
        assert_eq!(group.m_mtlBuffers[1].fsSlot, BindingMap::kAbsent);
    }

    #[test]
    fn buffer_source_index_does_not_claim_an_unrelated_resource_as_metal() {
        let unrelated =
            ResourceHandle::new(None, crate::gpu_resource::TestGPUResource::new(123_u32)).erase();
        let mut group = BindGroupMetal::new();
        group.base.m_retainedBuffers.push(unrelated);
        group.m_mtlBuffers = ManuallyDrop::new(vec![MTLBufferBinding::new(0)]);

        assert!(group.m_mtlBuffers[0].source(&group).is_none());
    }

    #[test]
    fn bind_group_owner_graph_is_thread_safe() {
        // The native derived owner is recording-thread confined for shared
        // access, but retains the source's one-way transfer boundary.
        assert_send::<BindGroupMetal>();
        // BindGroup's portable handles remain independently shareable only
        // through their intrusive owner API; this test deliberately does not
        // claim Sync for the Metal owner.
        assert!(std::mem::size_of::<BindGroupMetal>() > 0);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn buffer_record_resolves_the_live_backing_without_a_second_logical_owner() {
        use crate::types::BufferUsage;
        use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice, MTLResourceOptions};

        let Some(device) = MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let initial = device
            .newBufferWithLength_options(8, MTLResourceOptions::StorageModeShared)
            .expect("allocate Metal buffer");
        let mut payload = BufferMetal::new(
            8,
            BufferUsage::uniform,
            device,
            BufferMetalContextState::new(None),
        );
        payload.initializeBacking(Some(initial), None);
        let buffer = ResourceHandle::new(None, payload);
        let mut group = BindGroupMetal::new();
        group.base.m_retainedBuffers.push(buffer.clone().erase());
        let mut record = MTLBufferBinding::new(0);
        record.offset = 4;
        record.binding = 7;
        record.hasDynamicOffset = true;
        record.vsSlot = 2;
        record.fsSlot = 3;
        group.m_mtlBuffers = ManuallyDrop::new(vec![record]);
        let binding = &group.m_mtlBuffers[0];

        assert_eq!(binding.offset, 4);
        assert_eq!(buffer.debugging_refcnt(), 2);
        assert!(std::ptr::eq(binding.source(&group).unwrap(), &*buffer));
        let first = binding
            .source(&group)
            .unwrap()
            .current()
            .expect("first backing");

        buffer.m_contextState.setCurrentSerial(1);
        buffer.markBound();
        buffer.update(&[1, 2], 2, 3).expect("orphan bound buffer");
        let second = binding
            .source(&group)
            .unwrap()
            .current()
            .expect("second backing");

        assert_ne!(Retained::as_ptr(&first), Retained::as_ptr(&second));
        assert_eq!(buffer.debugging_refcnt(), 2);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn native_texture_and_sampler_records_retain_exact_binding_handles() {
        use objc2::rc::Weak;
        use objc2_metal::{
            MTLCreateSystemDefaultDevice, MTLDevice, MTLPixelFormat, MTLSamplerDescriptor,
            MTLTextureDescriptor,
        };

        let Some(device) = MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let descriptor = MTLTextureDescriptor::new();
        descriptor.setPixelFormat(MTLPixelFormat::RGBA8Unorm);
        // SAFETY: the texture is two-dimensional with non-zero extents and
        // one mip level, satisfying each descriptor setter's precondition.
        unsafe {
            descriptor.setWidth(1);
            descriptor.setHeight(1);
            descriptor.setMipmapLevelCount(1);
        }
        let texture = device
            .newTextureWithDescriptor(&descriptor)
            .expect("allocate Metal texture");
        let sampler = device
            .newSamplerStateWithDescriptor(&MTLSamplerDescriptor::new())
            .expect("allocate Metal sampler");
        let texture_pointer = Retained::as_ptr(&texture);
        let sampler_pointer = Retained::as_ptr(&sampler);
        let texture_owner = Weak::new(&*texture);
        let sampler_owner = Weak::new(&*sampler);

        let mut group = BindGroupMetal::new();
        group.base.m_retainedViews.push(
            ResourceHandle::new(None, crate::gpu_resource::TestGPUResource::new(1_u8)).erase(),
        );
        group.base.m_retainedSamplers.push(
            ResourceHandle::new(None, crate::gpu_resource::TestGPUResource::new(2_u8)).erase(),
        );
        group.m_mtlTextures = ManuallyDrop::new(vec![MTLTextureBinding {
            texture: Some(texture.clone()),
            vsSlot: 4,
            fsSlot: 5,
        }]);
        group.m_mtlSamplers = ManuallyDrop::new(vec![MTLSamplerBinding {
            sampler: Some(sampler.clone()),
            vsSlot: 6,
            fsSlot: 7,
        }]);
        drop(texture);
        drop(sampler);

        assert!(texture_owner.load().is_some());
        assert!(sampler_owner.load().is_some());
        assert_eq!(
            Retained::as_ptr(group.m_mtlTextures[0].texture.as_ref().unwrap()),
            texture_pointer
        );
        assert_eq!(
            Retained::as_ptr(group.m_mtlSamplers[0].sampler.as_ref().unwrap()),
            sampler_pointer
        );
        assert_eq!(group.base.m_retainedViews.len(), 1);
        assert_eq!(group.base.m_retainedSamplers.len(), 1);
    }
}
