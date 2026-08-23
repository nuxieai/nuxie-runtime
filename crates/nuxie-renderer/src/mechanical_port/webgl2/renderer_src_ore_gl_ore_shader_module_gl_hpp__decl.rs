//! Complete mechanical declaration translation of
//! `renderer/src/ore/gl/ore_shader_module_gl.hpp`.

#![allow(non_snake_case)]

use nuxie_ore_metal::gpu_resource::{GPUResource, GpuResourcePayload};
use nuxie_ore_metal::shader_module::ShaderModule;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

use super::gles3_decl::GLExecutionStamp;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_gl_ore_shader_module_gl.hpp");

#[repr(C)]
pub(crate) struct ShaderModuleGL {
    pub(crate) base: ManuallyDrop<ShaderModule>,
    pub(crate) m_glShader: u32,
    pub(crate) m_glShaderType: u32,
    /// Rust execution/lifetime sidecar after the complete source prefix.
    pub(crate) rust_execution: GLExecutionStamp,
}

impl ShaderModuleGL {
    pub(crate) fn new(execution: GLExecutionStamp) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_shader_module_backend_base()),
            m_glShader: 0,
            m_glShaderType: 0,
            rust_execution: execution,
        }
    }

    pub(crate) fn executionStamp(&self) -> &GLExecutionStamp {
        &self.rust_execution
    }
}
impl Deref for ShaderModuleGL {
    type Target = ShaderModule;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for ShaderModuleGL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
unsafe impl Send for ShaderModuleGL {}
unsafe impl GpuResourcePayload for ShaderModuleGL {
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
        assert_eq!(PINNED_SOURCE.lines().count(), 20);
        assert_eq!(std::mem::offset_of!(ShaderModuleGL, base), 0);
        assert!(
            std::mem::offset_of!(ShaderModuleGL, rust_execution)
                > std::mem::offset_of!(ShaderModuleGL, m_glShaderType)
        );
    }
}
