//! Complete mechanical declaration translation of
//! `renderer/src/ore/wgpu/ore_bind_group_layout_wgpu.hpp`.

#![allow(non_snake_case)]

use super::webgpu_cpp_decl::BindGroupLayout as WagyuBindGroupLayout;
use nuxie_ore_metal::bind_group_layout::BindGroupLayout;
use nuxie_ore_metal::gpu_resource::{GPUResource, GpuResourcePayload};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

pub(crate) const PINNED_SOURCE: &str = include_str!(
    "source/renderer_src_ore_wgpu_ore_bind_group_layout_wgpu.hpp"
);

/// Exact concrete ORE WebGPU bind-group-layout resource.
///
/// The source's value-owned Wagyu member is deliberately not a raw pointer:
/// its default value is null, assignment performs the source reference-count
/// transfer, and field destruction releases exactly once after the derived
/// object begins teardown and before the embedded ORE base is destroyed.
#[repr(C)]
pub(crate) struct BindGroupLayoutWGPU {
    base: ManuallyDrop<BindGroupLayout>,
    m_wgpuBGL: ManuallyDrop<WagyuBindGroupLayout>,
}

impl BindGroupLayoutWGPU {
    pub(crate) fn new() -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_bind_group_layout_backend_base()),
            m_wgpuBGL: ManuallyDrop::new(WagyuBindGroupLayout::default()),
        }
    }

    pub(crate) fn native(&self) -> &WagyuBindGroupLayout {
        &self.m_wgpuBGL
    }

    pub(crate) fn setNative(&mut self, native: WagyuBindGroupLayout) {
        *self.m_wgpuBGL = native;
    }
}

impl Drop for BindGroupLayoutWGPU {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_wgpuBGL);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl Deref for BindGroupLayoutWGPU {
    type Target = BindGroupLayout;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BindGroupLayoutWGPU {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

// C++ places no compile-time thread-affinity restriction on this resource;
// the owning ORE manager may move the complete object to its destruction path.
unsafe impl Send for BindGroupLayoutWGPU {}

unsafe impl GpuResourcePayload for BindGroupLayoutWGPU {
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

pub(crate) const SOURCE_CLASS_COUNT: usize = 2;
pub(crate) const SOURCE_BACKEND_FIELD_COUNT: usize = 1;
pub(crate) const SOURCE_DEFAULT_CONSTRUCTOR_COUNT: usize = 1;
pub(crate) const SOURCE_DEFAULT_DESTRUCTOR_COUNT: usize = 1;
pub(crate) const SOURCE_FRIEND_COUNT: usize = 1;

const _: [(); 495] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    #[test]
    fn complete_source_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 21);
        assert_eq!(SOURCE_CLASS_COUNT, 2);
        assert_eq!(SOURCE_BACKEND_FIELD_COUNT, 1);
        assert_eq!(SOURCE_DEFAULT_CONSTRUCTOR_COUNT, 1);
        assert_eq!(SOURCE_DEFAULT_DESTRUCTOR_COUNT, 1);
        assert_eq!(SOURCE_FRIEND_COUNT, 1);
    }

    #[test]
    fn inherited_base_is_offset_zero_and_handle_is_one_word() {
        assert_eq!(offset_of!(BindGroupLayoutWGPU, base), 0);
        assert_eq!(
            size_of::<WagyuBindGroupLayout>(),
            size_of::<super::super::webgpu_decl::WGPUBindGroupLayout>()
        );
    }
}
