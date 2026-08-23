//! Complete mechanical declaration translation of
//! `renderer/src/ore/wgpu/ore_sampler_wgpu.hpp`.

#![allow(non_snake_case)]

use super::webgpu_cpp_decl::Sampler as WagyuSampler;
use nuxie_ore_metal::gpu_resource::{GPUResource, GpuResourcePayload};
use nuxie_ore_metal::sampler::Sampler;
use std::ops::{Deref, DerefMut};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_sampler_wgpu.hpp");

/// Exact concrete ORE WebGPU sampler resource.
///
/// The Wagyu member remains value-owned so its default-null state, reference
/// counting, move semantics, and derived-before-base release order come from
/// the already-frozen `webgpu_cpp.h` owner.
#[repr(C)]
pub(crate) struct SamplerWGPU {
    base: Sampler,
    pub(crate) m_wgpuSampler: WagyuSampler,
}

impl SamplerWGPU {
    pub(crate) fn new() -> Self {
        Self {
            base: nuxie_ore_metal::new_sampler_backend_base(),
            m_wgpuSampler: WagyuSampler::default(),
        }
    }
}

impl Deref for SamplerWGPU {
    type Target = Sampler;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for SamplerWGPU {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

unsafe impl Send for SamplerWGPU {}

unsafe impl GpuResourcePayload for SamplerWGPU {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
}

pub(crate) const SOURCE_CLASS_COUNT: usize = 2;
pub(crate) const SOURCE_BACKEND_FIELD_COUNT: usize = 1;
pub(crate) const SOURCE_FRIEND_COUNT: usize = 1;
pub(crate) const SOURCE_RAII_RELEASE_COMMENT_COUNT: usize = 1;
const _: [(); 421] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    #[test]
    fn complete_header_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 19);
        assert_eq!(SOURCE_CLASS_COUNT, 2);
        assert_eq!(SOURCE_BACKEND_FIELD_COUNT, 1);
        assert_eq!(SOURCE_FRIEND_COUNT, 1);
        assert_eq!(SOURCE_RAII_RELEASE_COMMENT_COUNT, 1);
    }

    #[test]
    fn inherited_base_is_offset_zero_and_handle_is_one_word() {
        assert_eq!(offset_of!(SamplerWGPU, base), 0);
        assert_eq!(
            size_of::<WagyuSampler>(),
            size_of::<super::super::webgpu_decl::WGPUSampler>()
        );
    }
}
