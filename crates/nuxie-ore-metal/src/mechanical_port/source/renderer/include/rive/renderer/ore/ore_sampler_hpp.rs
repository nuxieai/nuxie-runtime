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
use core::mem::ManuallyDrop;

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
}

impl Drop for Sampler {
    fn drop(&mut self) {
        #[cfg(test)]
        super::super::gpu_resource_hpp::record_resource_drop_stage("Sampler.base");
        unsafe { ManuallyDrop::drop(&mut self.base) };
    }
}

unsafe impl GpuResourcePayload for Sampler {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base
    }
}

impl Sampler {
    // public:

    // virtual ~Sampler() = default;
    // Rust's default drop glue supplies the virtual-destructor boundary for
    // the concrete resource owner; no extra state is introduced here.

    // protected:
    // friend class Context;
    // friend class RenderPass;
    // Rust has no friend declarations; these source access boundaries remain
    // visible here, and the owning translation units use crate visibility.

    // Sampler() : rive::gpu::GPUResource(nullptr) {}
    pub(crate) fn new() -> Self {
        Self {
            base: ManuallyDrop::new(GPUResource::new(None)),
        }
    }

    // Sampler(rcp<rive::gpu::GPUResourceManager> manager) :
    //     rive::gpu::GPUResource(std::move(manager))
    // {}
    // The outer concrete `ResourceHandle` owns the optional manager.
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
