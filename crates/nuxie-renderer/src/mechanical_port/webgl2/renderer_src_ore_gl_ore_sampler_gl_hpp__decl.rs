//! Complete mechanical declaration translation of
//! `renderer/src/ore/gl/ore_sampler_gl.hpp`.

#![allow(non_snake_case)]

use nuxie_ore_metal::gpu_resource::{GPUResource, GpuResourcePayload};
use nuxie_ore_metal::sampler::Sampler;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

use super::gles3_decl::GLExecutionStamp;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_gl_ore_sampler_gl.hpp");

#[repr(C)]
pub(crate) struct SamplerGL {
    pub(crate) base: ManuallyDrop<Sampler>,
    pub(crate) m_glSampler: u32,
    /// Rust execution/lifetime sidecar after the complete source prefix.
    pub(crate) rust_execution: GLExecutionStamp,
}

impl SamplerGL {
    pub(crate) fn new(execution: GLExecutionStamp) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_sampler_backend_base()),
            m_glSampler: 0,
            rust_execution: execution,
        }
    }

    pub(crate) fn executionStamp(&self) -> &GLExecutionStamp {
        &self.rust_execution
    }
}
impl Deref for SamplerGL {
    type Target = Sampler;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for SamplerGL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
unsafe impl Send for SamplerGL {}
unsafe impl GpuResourcePayload for SamplerGL {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_header_denominator_is_frozen() {
        assert_eq!(PINNED_SOURCE.lines().count(), 18);
        assert_eq!(std::mem::offset_of!(SamplerGL, base), 0);
        assert!(
            std::mem::offset_of!(SamplerGL, rust_execution)
                > std::mem::offset_of!(SamplerGL, m_glSampler)
        );
    }
}
