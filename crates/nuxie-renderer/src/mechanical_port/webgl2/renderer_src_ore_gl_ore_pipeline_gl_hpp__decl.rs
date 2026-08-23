//! Complete mechanical declaration translation of
//! `renderer/src/ore/gl/ore_pipeline_gl.hpp`.

#![allow(non_snake_case)]

use nuxie_ore_metal::gpu_resource::{GPUResource, GpuResourcePayload};
use nuxie_ore_metal::pipeline::Pipeline;
use nuxie_ore_metal::types::PipelineDesc;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

use super::gles3_decl::GLExecutionStamp;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_gl_ore_pipeline_gl.hpp");

#[repr(C)]
pub(crate) struct PipelineGL {
    pub(crate) base: ManuallyDrop<Pipeline>,
    pub(crate) m_glProgram: u32,
    /// Rust execution/lifetime sidecar after the complete source prefix.
    pub(crate) rust_execution: GLExecutionStamp,
}

impl PipelineGL {
    pub(crate) fn new(desc: &PipelineDesc<'_>, execution: GLExecutionStamp) -> Option<Self> {
        Some(Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_pipeline_backend_base_without_manager(
                desc,
            )?),
            m_glProgram: 0,
            rust_execution: execution,
        })
    }

    pub(crate) fn executionStamp(&self) -> &GLExecutionStamp {
        &self.rust_execution
    }
}

impl Deref for PipelineGL {
    type Target = Pipeline;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for PipelineGL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
unsafe impl Send for PipelineGL {}
unsafe impl GpuResourcePayload for PipelineGL {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
    fn pipeline_base(&self) -> Option<&Pipeline> {
        Some(&self.base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_header_denominator_is_frozen() {
        assert_eq!(PINNED_SOURCE.lines().count(), 20);
        assert_eq!(std::mem::offset_of!(PipelineGL, base), 0);
        assert!(
            std::mem::offset_of!(PipelineGL, rust_execution)
                > std::mem::offset_of!(PipelineGL, m_glProgram)
        );
        assert!(std::mem::size_of::<PipelineGL>() > std::mem::size_of::<Pipeline>() + 4);
    }
}
