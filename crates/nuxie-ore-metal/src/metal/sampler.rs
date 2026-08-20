// Mechanical translation of:
// - renderer/src/ore/metal/ore_sampler_metal.hpp
// - renderer/src/ore/metal/ore_sampler_metal.mm
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

#![allow(non_snake_case)]

use std::any::Any;

use crate::gpu_resource::{GpuResourceManager, ResourceHandle};
use crate::sampler::Sampler;
use crate::types::{BackendId, Sampler as SamplerResource};

use super::MetalBackend;

#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2::rc::Retained;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2::runtime::ProtocolObject;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2_metal::MTLSamplerState;

#[cfg(any(target_os = "ios", target_os = "macos"))]
struct RetainedMetalSampler(Retained<ProtocolObject<dyn MTLSamplerState>>);

// SAFETY: MTLSamplerState is immutable after creation and supports concurrent
// retain/release and binding. The wrapper exposes only shared protocol access.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Send for RetainedMetalSampler {}
// SAFETY: Same invariant as the `Send` implementation above.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Sync for RetainedMetalSampler {}

/// Direct translation of `rive::ore::SamplerMetal`.
pub struct SamplerMetal {
    base: Sampler,
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    m_mtlSampler: Option<RetainedMetalSampler>,
}

impl SamplerMetal {
    /// Translate the default nil native state.
    pub fn new() -> Self {
        Self {
            base: Sampler::new(),
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            m_mtlSampler: None,
        }
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub fn with_native_sampler_state(state: Retained<ProtocolObject<dyn MTLSamplerState>>) -> Self {
        Self {
            base: Sampler::new(),
            m_mtlSampler: Some(RetainedMetalSampler(state)),
        }
    }

    pub fn base(&self) -> &Sampler {
        &self.base
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub fn mtlSampler(&self) -> Option<&ProtocolObject<dyn MTLSamplerState>> {
        self.m_mtlSampler.as_ref().map(|sampler| sampler.0.as_ref())
    }

    pub fn into_resource(self, manager: Option<GpuResourceManager>) -> ResourceHandle<Self> {
        ResourceHandle::new(manager, self)
    }
}

impl Default for SamplerMetal {
    fn default() -> Self {
        Self::new()
    }
}

impl SamplerResource for SamplerMetal {
    fn backend_id(&self) -> BackendId {
        BackendId::of::<MetalBackend>()
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_resource::GpuResourceManagerOwner;

    #[test]
    fn resource_handle_is_the_only_manager_owner() {
        let owner = GpuResourceManagerOwner::new();
        let handle = SamplerMetal::new().into_resource(Some(owner.manager()));
        assert!(handle.manager().is_some());
        assert_eq!(std::mem::size_of_val(handle.base()), 0);
        let clone = handle.clone();
        assert!(handle.ptr_eq(&clone));
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_sampler_retains_the_native_state() {
        use objc2_metal::{
            MTLCreateSystemDefaultDevice, MTLDevice, MTLSamplerDescriptor, MTLSamplerMinMagFilter,
        };

        let Some(device) = MTLCreateSystemDefaultDevice() else {
            return;
        };
        let descriptor = MTLSamplerDescriptor::new();
        descriptor.setMinFilter(MTLSamplerMinMagFilter::Linear);
        let state = device
            .newSamplerStateWithDescriptor(&descriptor)
            .expect("create Metal sampler state");
        let sampler = SamplerMetal::with_native_sampler_state(state);
        assert!(sampler.mtlSampler().is_some());
    }
}
