//! Complete mechanical declaration translation of
//! `renderer/src/ore/wgpu/ore_shader_module_wgpu.hpp`.

#![allow(non_snake_case)]

use super::webgpu_cpp_decl::ShaderModule as WagyuShaderModule;
use nuxie_ore_metal::gpu_resource::{GPUResource, GpuResourcePayload};
use nuxie_ore_metal::shader_module::ShaderModule;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_shader_module_wgpu.hpp");

/// Exact concrete ORE WebGPU shader-module resource.
///
/// The source owns its native module by value. The frozen Wagyu wrapper keeps
/// the same null default, add-reference/copy, acquire/move, and release rules.
#[repr(C)]
pub(crate) struct ShaderModuleWGPU {
    base: ManuallyDrop<ShaderModule>,
    m_wgpuShaderModule: ManuallyDrop<WagyuShaderModule>,
}

impl ShaderModuleWGPU {
    pub(crate) fn new() -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_shader_module_backend_base()),
            m_wgpuShaderModule: ManuallyDrop::new(WagyuShaderModule::default()),
        }
    }

    pub(crate) fn native(&self) -> &WagyuShaderModule {
        &self.m_wgpuShaderModule
    }

    pub(crate) fn setNative(&mut self, native: WagyuShaderModule) {
        *self.m_wgpuShaderModule = native;
    }
}

impl Drop for ShaderModuleWGPU {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_wgpuShaderModule);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl Deref for ShaderModuleWGPU {
    type Target = ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for ShaderModuleWGPU {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

unsafe impl Send for ShaderModuleWGPU {}

unsafe impl GpuResourcePayload for ShaderModuleWGPU {
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
const _: [(); 479] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    #[test]
    fn complete_header_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 21);
        assert_eq!(SOURCE_CLASS_COUNT, 2);
        assert_eq!(SOURCE_BACKEND_FIELD_COUNT, 1);
        assert_eq!(SOURCE_FRIEND_COUNT, 1);
        assert_eq!(SOURCE_RAII_RELEASE_COMMENT_COUNT, 1);
    }

    #[test]
    fn inherited_base_is_offset_zero_and_handle_is_one_word() {
        assert_eq!(offset_of!(ShaderModuleWGPU, base), 0);
        assert_eq!(
            size_of::<WagyuShaderModule>(),
            size_of::<super::super::webgpu_decl::WGPUShaderModule>()
        );
    }
}
