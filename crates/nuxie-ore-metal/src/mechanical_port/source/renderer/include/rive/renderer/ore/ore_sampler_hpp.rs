/*
 * Copyright 2025 Rive
 */

// #pragma once

// #include "rive/renderer/gpu_resource.hpp"
// #include "utils/lite_rtti.hpp"
// #include "rive/renderer/ore/ore_types.hpp"

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/ore/ore_sampler.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::super::gpu_resource_hpp::AnyResourceHandle;
use super::ore_types_hpp::{CompareFunction, Filter, SamplerDesc};
use core::mem::ManuallyDrop;
use std::cell::Cell;

use super::super::gpu_resource_hpp::{GPUResource, GpuResourcePayload};
// namespace rive::ore

// class Context;
// The source forward declaration is retained for the friend relationships
// below. Context is owned by its own translation unit.

// class Sampler : public rive::gpu::GPUResource,
//                 public ENABLE_LITE_RTTI(Sampler)
//
// Rust has no class inheritance. The first field is the GPUResource base
// subobject, preserving the source base-before-members layout and destruction
// relationship. The second source base, `ENABLE_LITE_RTTI(Sampler)`, is owned
// by the generic lite-RTTI translation and is deliberately not duplicated as
// a payload field here; concrete backends use its override seam.
#[repr(C)]
pub struct Sampler {
    pub(crate) base: ManuallyDrop<GPUResource>,
    described: Cell<bool>,
    filtering: Cell<bool>,
    comparison: Cell<bool>,
}

impl Drop for Sampler {
    fn drop(&mut self) {
        #[cfg(test)]
        super::super::gpu_resource_hpp::record_resource_drop_stage("Sampler.base");
        unsafe { ManuallyDrop::drop(&mut self.base) };
    }
}

unsafe impl GpuResourcePayload for Sampler {
    fn sampler_base(&self) -> Option<&Sampler> {
        Some(self)
    }
    fn gpu_resource(&self) -> &GPUResource {
        &self.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base
    }
}

impl Sampler {
    // Intrusive aliases share metadata; resource payload access is confined to
    // the recording thread by the retained handle's base projection.
    pub fn recordDesc(&self, desc: &SamplerDesc<'_>) {
        self.filtering.set(
            desc.minFilter == Filter::linear
                || desc.magFilter == Filter::linear
                || desc.mipmapFilter == Filter::linear,
        );
        self.comparison.set(desc.compare != CompareFunction::none);
        self.described.set(true);
    }
    pub fn described(&self) -> bool {
        self.described.get()
    }
    pub fn filtering(&self) -> bool {
        self.filtering.get()
    }
    pub fn comparison(&self) -> bool {
        self.comparison.get()
    }
    // public:

    // virtual ~Sampler() = default;
    // Rust's drop glue supplies the virtual-destructor boundary for the
    // concrete resource owner.

    // protected:
    // friend class Context;
    // friend class RenderPass;
    // Rust has no friend declarations; these source access boundaries remain
    // visible here, and the owning translation units use crate visibility.

    // Sampler() : rive::gpu::GPUResource(nullptr) {}
    pub(crate) fn new() -> Self {
        Self {
            base: ManuallyDrop::new(GPUResource::new(None)),
            described: Cell::new(false),
            filtering: Cell::new(false),
            comparison: Cell::new(false),
        }
    }

    // Sampler(rcp<rive::gpu::GPUResourceManager> manager) :
    //     rive::gpu::GPUResource(std::move(manager))
    // {}
    // The outer concrete `ResourceHandle` owns the optional manager.
}

pub fn describedSampler(
    sampler: Option<AnyResourceHandle>,
    desc: &SamplerDesc<'_>,
) -> Option<AnyResourceHandle> {
    if let Some(sampler) = &sampler {
        sampler
            .samplerBase()
            .expect("Sampler base")
            .recordDesc(desc);
    }
    sampler
}

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampler_payload_has_no_duplicate_resource_owner() {
        let _sampler = Sampler::new();
        assert_eq!(core::mem::offset_of!(Sampler, base), 0);
    }
}
