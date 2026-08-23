//! Complete mechanical declaration translation of
//! `renderer/src/ore/gl/ore_bind_group_gl.hpp`.

#![allow(non_snake_case)]

use nuxie_ore_metal::bind_group::BindGroup;
use nuxie_ore_metal::bind_group_layout::BindGroupLayout;
use nuxie_ore_metal::gpu_resource::{GPUResource, GpuResourcePayload};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

use super::gles3_decl::GLExecutionStamp;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_gl_ore_bind_group_gl.hpp");

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GLUBOBinding {
    pub(crate) buffer: u32,
    pub(crate) offset: u32,
    pub(crate) size: u32,
    pub(crate) binding: u32,
    pub(crate) slot: u32,
    pub(crate) hasDynamicOffset: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GLTexBinding {
    pub(crate) texture: u32,
    pub(crate) target: u32,
    pub(crate) slot: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GLSamplerBinding {
    pub(crate) sampler: u32,
    pub(crate) slot: u32,
}

#[repr(C)]
pub(crate) struct BindGroupGL {
    pub(crate) base: ManuallyDrop<BindGroup>,
    pub(crate) m_glUBOs: ManuallyDrop<Vec<GLUBOBinding>>,
    pub(crate) m_glTextures: ManuallyDrop<Vec<GLTexBinding>>,
    pub(crate) m_glSamplers: ManuallyDrop<Vec<GLSamplerBinding>>,
    /// Rust execution/lifetime sidecar after the complete source prefix.
    pub(crate) rust_execution: GLExecutionStamp,
}

impl BindGroupGL {
    pub(crate) fn new(execution: GLExecutionStamp) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_bind_group_backend_base_without_manager()),
            m_glUBOs: ManuallyDrop::new(Vec::new()),
            m_glTextures: ManuallyDrop::new(Vec::new()),
            m_glSamplers: ManuallyDrop::new(Vec::new()),
            rust_execution: execution,
        }
    }

    pub(crate) fn executionStamp(&self) -> &GLExecutionStamp {
        &self.rust_execution
    }
}

impl Drop for BindGroupGL {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_glSamplers);
            ManuallyDrop::drop(&mut self.m_glTextures);
            ManuallyDrop::drop(&mut self.m_glUBOs);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl Deref for BindGroupGL {
    type Target = BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BindGroupGL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

unsafe impl Send for BindGroupGL {}

unsafe impl GpuResourcePayload for BindGroupGL {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
}

/// WebGL execution identity carried beside the source's backend-independent
/// `BindGroupLayout`. The source has no GL-native layout name, so the entire
/// `BindGroupLayout` remains the exact prefix and this stamp is metadata only.
#[repr(C)]
pub(crate) struct BindGroupLayoutGL {
    pub(crate) base: BindGroupLayout,
    pub(crate) rust_execution: GLExecutionStamp,
}

impl BindGroupLayoutGL {
    pub(crate) fn new(execution: GLExecutionStamp) -> Self {
        Self {
            base: nuxie_ore_metal::new_bind_group_layout_backend_base(),
            rust_execution: execution,
        }
    }

    pub(crate) fn executionStamp(&self) -> &GLExecutionStamp {
        &self.rust_execution
    }
}

impl Deref for BindGroupLayoutGL {
    type Target = BindGroupLayout;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BindGroupLayoutGL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

unsafe impl Send for BindGroupLayoutGL {}

unsafe impl GpuResourcePayload for BindGroupLayoutGL {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }

    fn bind_group_layout_base(&self) -> Option<&BindGroupLayout> {
        Some(&self.base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::offset_of;

    #[test]
    fn complete_bind_group_field_denominator_and_base_layout_are_frozen() {
        assert_eq!(PINNED_SOURCE.lines().count(), 45);
        assert_eq!(offset_of!(BindGroupGL, base), 0);
        assert_eq!(offset_of!(GLUBOBinding, buffer), 0);
        assert!(offset_of!(GLUBOBinding, hasDynamicOffset) > offset_of!(GLUBOBinding, slot));
        assert_eq!(std::mem::size_of::<GLTexBinding>(), 12);
        assert_eq!(std::mem::size_of::<GLSamplerBinding>(), 8);
        assert!(offset_of!(BindGroupGL, rust_execution) > offset_of!(BindGroupGL, m_glSamplers));
        assert!(std::mem::size_of::<BindGroupGL>() > std::mem::size_of::<BindGroup>());
        assert_eq!(offset_of!(BindGroupLayoutGL, base), 0);
        assert!(
            offset_of!(BindGroupLayoutGL, rust_execution) > offset_of!(BindGroupLayoutGL, base)
        );
    }
}
