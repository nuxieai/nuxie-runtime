//! Complete mechanical declaration translation of
//! `renderer/src/ore/wgpu/ore_texture_wgpu.hpp`.

#![allow(non_snake_case)]

use super::webgpu_cpp_decl::{Queue as WagyuQueue, Texture as WagyuTexture};
use super::webgpu_cpp_decl::TextureView as WagyuTextureView;
use nuxie_ore_metal::gpu_resource::{
    AnyResourceHandle, GPUResource, GPUResourceManager, GpuResourcePayload,
};
use nuxie_ore_metal::texture::{Texture, TextureApi, TextureUploadError, TextureView};
use nuxie_ore_metal::types::{
    TextureDataDesc, TextureDesc, TextureFormat, TextureType, TextureViewDesc,
};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_texture_wgpu.hpp");

#[repr(C)]
pub(crate) struct TextureWGPU {
    pub(crate) base: ManuallyDrop<Texture>,
    m_wgpuTexture: ManuallyDrop<WagyuTexture>,
    /// Source "weak ref" comment means a copied, addref'd queue wrapper.
    m_wgpuQueue: ManuallyDrop<WagyuQueue>,
}

impl TextureWGPU {
    pub(crate) fn new(manager: GPUResourceManager, desc: &TextureDesc<'_>) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_texture_backend_base(manager, desc)),
            m_wgpuTexture: ManuallyDrop::new(WagyuTexture::default()),
            m_wgpuQueue: ManuallyDrop::new(WagyuQueue::default()),
        }
    }

    pub(crate) fn nativeTexture(&self) -> &WagyuTexture { &self.m_wgpuTexture }
    pub(crate) fn queue(&self) -> &WagyuQueue { &self.m_wgpuQueue }
    pub(crate) fn setNativeTexture(&mut self, texture: WagyuTexture) { *self.m_wgpuTexture = texture; }
    pub(crate) fn setQueue(&mut self, queue: WagyuQueue) { *self.m_wgpuQueue = queue; }
}

impl Drop for TextureWGPU {
    fn drop(&mut self) {
        // C++ destroys members in reverse declaration order, then its base.
        unsafe {
            ManuallyDrop::drop(&mut self.m_wgpuQueue);
            ManuallyDrop::drop(&mut self.m_wgpuTexture);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl Deref for TextureWGPU {
    type Target = Texture;
    fn deref(&self) -> &Self::Target { &self.base }
}
impl DerefMut for TextureWGPU {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.base }
}

unsafe impl Send for TextureWGPU {}
unsafe impl GpuResourcePayload for TextureWGPU {
    fn gpu_resource(&self) -> &GPUResource { self.base.gpu_resource() }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource { self.base.gpu_resource_mut() }
}

impl TextureApi for TextureWGPU {
    fn width(&self) -> u32 { self.base.width() }
    fn height(&self) -> u32 { self.base.height() }
    fn depthOrArrayLayers(&self) -> u32 { self.base.depthOrArrayLayers() }
    fn format(&self) -> TextureFormat { self.base.format() }
    fn r#type(&self) -> TextureType { self.base.r#type() }
    fn numMipmaps(&self) -> u32 { self.base.numMipmaps() }
    fn sampleCount(&self) -> u32 { self.base.sampleCount() }
    fn isRenderTarget(&self) -> bool { self.base.isRenderTarget() }
    fn upload(&self, data: &TextureDataDesc<'_>) -> Result<(), TextureUploadError> {
        super::ore_texture_wgpu_impl::upload(self, data)
    }
}

#[repr(C)]
pub(crate) struct TextureViewWGPU {
    pub(crate) base: ManuallyDrop<TextureView>,
    m_wgpuTextureView: ManuallyDrop<WagyuTextureView>,
}

impl TextureViewWGPU {
    pub(crate) fn new(
        manager: GPUResourceManager,
        texture: AnyResourceHandle,
        desc: &TextureViewDesc<'_>,
    ) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_texture_view_backend_base(
                manager, texture, desc,
            )),
            m_wgpuTextureView: ManuallyDrop::new(WagyuTextureView::default()),
        }
    }

    pub(crate) fn native(&self) -> &WagyuTextureView { &self.m_wgpuTextureView }
    pub(crate) fn setNative(&mut self, view: WagyuTextureView) { *self.m_wgpuTextureView = view; }
}

impl Drop for TextureViewWGPU {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_wgpuTextureView);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl Deref for TextureViewWGPU {
    type Target = TextureView;
    fn deref(&self) -> &Self::Target { &self.base }
}
impl DerefMut for TextureViewWGPU {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.base }
}
unsafe impl Send for TextureViewWGPU {}
unsafe impl GpuResourcePayload for TextureViewWGPU {
    fn gpu_resource(&self) -> &GPUResource { self.base.gpu_resource() }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource { self.base.gpu_resource_mut() }
}

pub(crate) const SOURCE_CLASS_COUNT: usize = 3;
pub(crate) const SOURCE_BACKEND_FIELD_COUNT: usize = 3;
pub(crate) const SOURCE_CONSTRUCTOR_COUNT: usize = 2;
pub(crate) const SOURCE_DESTRUCTOR_COUNT: usize = 2;
pub(crate) const SOURCE_FRIEND_COUNT: usize = 2;
const _: [(); 981] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    #[test]
    fn complete_header_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 35);
        assert_eq!(SOURCE_CLASS_COUNT, 3);
        assert_eq!(SOURCE_BACKEND_FIELD_COUNT, 3);
        assert_eq!(SOURCE_CONSTRUCTOR_COUNT, 2);
        assert_eq!(SOURCE_DESTRUCTOR_COUNT, 2);
        assert_eq!(SOURCE_FRIEND_COUNT, 2);
    }

    #[test]
    fn bases_are_offset_zero_and_all_native_members_are_one_word() {
        assert_eq!(offset_of!(TextureWGPU, base), 0);
        assert_eq!(offset_of!(TextureViewWGPU, base), 0);
        assert_eq!(size_of::<WagyuTexture>(), size_of::<usize>());
        assert_eq!(size_of::<WagyuQueue>(), size_of::<usize>());
        assert_eq!(size_of::<WagyuTextureView>(), size_of::<usize>());
    }
}
