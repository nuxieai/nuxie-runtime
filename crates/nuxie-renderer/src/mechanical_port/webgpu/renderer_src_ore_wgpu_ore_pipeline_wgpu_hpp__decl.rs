//! Complete mechanical declaration translation of
//! `renderer/src/ore/wgpu/ore_pipeline_wgpu.hpp`.

#![allow(non_snake_case)]

use super::webgpu_cpp_decl::{
    Device as WagyuDevice, PipelineLayout as WagyuPipelineLayout,
    RenderPipeline as WagyuRenderPipeline,
};
use nuxie_ore_metal::gpu_resource::{GPUResource, GPUResourceManager, GpuResourcePayload};
use nuxie_ore_metal::pipeline::Pipeline;
use nuxie_ore_metal::types::PipelineDesc;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_pipeline_wgpu.hpp");

#[repr(C)]
pub(crate) struct PipelineWGPU {
    pub(crate) base: ManuallyDrop<Pipeline>,
    m_wgpuDevice: ManuallyDrop<WagyuDevice>,
    m_wgpuPipeline: ManuallyDrop<WagyuRenderPipeline>,
    m_wgpuPipelineLayout: ManuallyDrop<WagyuPipelineLayout>,
}

impl PipelineWGPU {
    pub(crate) fn new(manager: GPUResourceManager, desc: &PipelineDesc<'_>) -> Option<Self> {
        Some(Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_pipeline_backend_base(manager, desc)?),
            m_wgpuDevice: ManuallyDrop::new(WagyuDevice::default()),
            m_wgpuPipeline: ManuallyDrop::new(WagyuRenderPipeline::default()),
            m_wgpuPipelineLayout: ManuallyDrop::new(WagyuPipelineLayout::default()),
        })
    }

    pub(crate) fn device(&self) -> &WagyuDevice {
        &self.m_wgpuDevice
    }

    pub(crate) fn nativePipeline(&self) -> &WagyuRenderPipeline {
        &self.m_wgpuPipeline
    }

    pub(crate) fn nativeLayout(&self) -> &WagyuPipelineLayout {
        &self.m_wgpuPipelineLayout
    }

    pub(crate) fn setDevice(&mut self, device: WagyuDevice) {
        *self.m_wgpuDevice = device;
    }

    pub(crate) fn setNativePipeline(&mut self, pipeline: WagyuRenderPipeline) {
        *self.m_wgpuPipeline = pipeline;
    }

    pub(crate) fn setNativeLayout(&mut self, layout: WagyuPipelineLayout) {
        *self.m_wgpuPipelineLayout = layout;
    }
}

impl Drop for PipelineWGPU {
    fn drop(&mut self) {
        // C++ destroys derived members in reverse declaration order, then its
        // Pipeline base; each Wagyu owner performs one conditional Release.
        unsafe {
            ManuallyDrop::drop(&mut self.m_wgpuPipelineLayout);
            ManuallyDrop::drop(&mut self.m_wgpuPipeline);
            ManuallyDrop::drop(&mut self.m_wgpuDevice);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl Deref for PipelineWGPU {
    type Target = Pipeline;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for PipelineWGPU {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

// The source permits the ORE manager to reclaim the complete immutable
// pipeline payload on its deferred destruction path.
unsafe impl Send for PipelineWGPU {}

unsafe impl GpuResourcePayload for PipelineWGPU {
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

pub(crate) const SOURCE_CLASS_COUNT: usize = 2;
pub(crate) const SOURCE_BACKEND_FIELD_COUNT: usize = 3;
pub(crate) const SOURCE_CONSTRUCTOR_COUNT: usize = 1;
pub(crate) const SOURCE_DEFAULT_DESTRUCTOR_COUNT: usize = 1;
pub(crate) const SOURCE_FRIEND_COUNT: usize = 2;
const _: [(); 588] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    #[test]
    fn complete_header_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 22);
        assert_eq!(SOURCE_CLASS_COUNT, 2);
        assert_eq!(SOURCE_BACKEND_FIELD_COUNT, 3);
        assert_eq!(SOURCE_CONSTRUCTOR_COUNT, 1);
        assert_eq!(SOURCE_DEFAULT_DESTRUCTOR_COUNT, 1);
        assert_eq!(SOURCE_FRIEND_COUNT, 2);
    }

    #[test]
    fn base_is_offset_zero_and_each_wagyu_owner_is_one_handle() {
        assert_eq!(offset_of!(PipelineWGPU, base), 0);
        assert_eq!(size_of::<WagyuDevice>(), size_of::<usize>());
        assert_eq!(size_of::<WagyuRenderPipeline>(), size_of::<usize>());
        assert_eq!(size_of::<WagyuPipelineLayout>(), size_of::<usize>());
    }
}
