/*
 * Copyright 2025 Rive
 */

// #pragma once
// #include "rive/renderer/ore/ore_sampler.hpp"
// #import <Metal/Metal.h>

// Mechanical translation of the complete pinned source header
// renderer/src/ore/metal/ore_sampler_metal.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::{
    GPUResource, GpuResourcePayload,
};
use std::mem::ManuallyDrop;

// `id<MTLSamplerState>` is a nullable, strong Objective-C owner under ARC.
// Rust's `Retained<T>` is the corresponding strong owner; `Option` preserves
// the source `nil` state. The non-Apple stand-in keeps this source-shaped
// translation available to tools that inspect it off Apple.
#[cfg(target_vendor = "apple")]
use objc2::rc::Retained;
#[cfg(target_vendor = "apple")]
use objc2::runtime::ProtocolObject;
#[cfg(target_vendor = "apple")]
use objc2_metal::MTLSamplerState;

#[cfg(target_vendor = "apple")]
type NativeMetalSampler = Option<Retained<ProtocolObject<dyn MTLSamplerState>>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalSampler = Option<()>;

// namespace rive::ore

// class ContextMetal;
// The source forward declaration is retained for the friend relationship
// below. ContextMetal is owned by its own translation unit.

// class SamplerMetal : public LITE_RTTI_OVERRIDE(Sampler, SamplerMetal)
// {
// Rust has no class inheritance. `base` is the first field to preserve the
// source Sampler base-subobject order. `LITE_RTTI_OVERRIDE(Sampler,
// SamplerMetal)` remains the source lite-RTTI identity/override seam and is
// not duplicated as a payload field.
#[repr(C)]
pub struct SamplerMetal {
    pub(crate) base: ManuallyDrop<Sampler>,
    // private:
    // friend class ContextMetal;
    // Rust has no friend declarations; this source access boundary remains
    // visible here, and the owning translation unit uses crate visibility.
    // id<MTLSamplerState> m_mtlSampler = nil;
    // `NativeMetalSampler` retains the non-nil Objective-C sampler state
    // until the enclosing logical SamplerMetal owner is dropped.
    pub(crate) m_mtlSampler: ManuallyDrop<NativeMetalSampler>,
}

// SAFETY: MTLSamplerState is immutable after publication and supports
// concurrent retain/release and binding.
unsafe impl Send for SamplerMetal {}

unsafe impl GpuResourcePayload for SamplerMetal {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base.base
    }
}

impl SamplerMetal {
    // public:

    // SamplerMetal() = default;
    pub(crate) fn new() -> Self {
        Self {
            base: ManuallyDrop::new(Sampler::new()),
            m_mtlSampler: ManuallyDrop::new(None),
        }
    }

    // ~SamplerMetal() override = default; // ARC releases m_mtlSampler
    // Rust's default drop glue releases the retained native sampler owner
    // before the remaining source-shaped fields.
}

impl Drop for SamplerMetal {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_mtlSampler);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_resource::GPUResourceManagerOwner;

    #[test]
    fn resource_handle_is_the_only_manager_owner() {
        let owner = GPUResourceManagerOwner::new();
        let handle =
            crate::gpu_resource::ResourceHandle::new(Some(owner.manager()), SamplerMetal::new());
        assert!(handle.manager().is_some());
        let clone = handle.clone();
        assert!(handle.ptr_eq(&clone));
        drop(clone);
        drop(handle);
        owner.shutdown();
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn live_sampler_retains_the_native_state() {
        use objc2_metal::{
            MTLCreateSystemDefaultDevice, MTLDevice, MTLSamplerDescriptor, MTLSamplerMinMagFilter,
        };

        let Some(device) = MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let descriptor = MTLSamplerDescriptor::new();
        descriptor.setMinFilter(MTLSamplerMinMagFilter::Linear);
        let state = device
            .newSamplerStateWithDescriptor(&descriptor)
            .expect("create Metal sampler state");
        let mut sampler = SamplerMetal::new();
        sampler.m_mtlSampler = ManuallyDrop::new(Some(state));
        assert!(sampler.m_mtlSampler.is_some());
    }
}
