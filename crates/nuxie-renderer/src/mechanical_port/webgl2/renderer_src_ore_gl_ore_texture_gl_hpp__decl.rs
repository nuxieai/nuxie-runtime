//! Complete mechanical declaration translation of
//! `renderer/src/ore/gl/ore_texture_gl.hpp`.

#![allow(non_snake_case)]

use nuxie_ore_metal::gpu_resource::{AnyResourceHandle, GPUResource, GpuResourcePayload};
use nuxie_ore_metal::texture::{Texture, TextureApi, TextureUploadError, TextureView};
use nuxie_ore_metal::types::{
    TextureDataDesc, TextureDesc, TextureFormat, TextureType, TextureViewDesc,
};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

use super::gles3_decl::GLExecutionStamp;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_gl_ore_texture_gl.hpp");

#[repr(C)]
pub(crate) struct TextureGL {
    pub(crate) base: ManuallyDrop<Texture>,
    pub(crate) m_glTexture: u32,
    pub(crate) m_glRenderbuffer: u32,
    pub(crate) m_glTarget: u32,
    pub(crate) m_glOwnsTexture: bool,
    /// Rust execution/lifetime sidecar after the complete source prefix.
    pub(crate) rust_execution: GLExecutionStamp,
}

impl TextureGL {
    pub(crate) fn new(desc: &TextureDesc<'_>, execution: GLExecutionStamp) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_texture_backend_base_without_manager(
                desc,
            )),
            m_glTexture: 0,
            m_glRenderbuffer: 0,
            m_glTarget: 0,
            m_glOwnsTexture: false,
            rust_execution: execution,
        }
    }

    pub(crate) fn executionStamp(&self) -> &GLExecutionStamp {
        &self.rust_execution
    }
}

impl Deref for TextureGL {
    type Target = Texture;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for TextureGL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
unsafe impl GpuResourcePayload for TextureGL {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
}
impl TextureApi for TextureGL {
    fn width(&self) -> u32 {
        self.base.width()
    }
    fn height(&self) -> u32 {
        self.base.height()
    }
    fn depthOrArrayLayers(&self) -> u32 {
        self.base.depthOrArrayLayers()
    }
    fn format(&self) -> TextureFormat {
        self.base.format()
    }
    fn r#type(&self) -> TextureType {
        self.base.r#type()
    }
    fn numMipmaps(&self) -> u32 {
        self.base.numMipmaps()
    }
    fn sampleCount(&self) -> u32 {
        self.base.sampleCount()
    }
    fn isRenderTarget(&self) -> bool {
        self.base.isRenderTarget()
    }
    fn upload(&self, data: &TextureDataDesc<'_>) -> Result<(), TextureUploadError> {
        super::ore_texture_gl_impl::upload(self, data)
    }
}

#[repr(C)]
pub(crate) struct TextureViewGL {
    pub(crate) base: ManuallyDrop<TextureView>,
    pub(crate) m_glTextureView: u32,
    /// Rust execution/lifetime sidecar after the complete source prefix.
    pub(crate) rust_execution: GLExecutionStamp,
}

impl TextureViewGL {
    pub(crate) fn new(
        texture: AnyResourceHandle,
        desc: &TextureViewDesc<'_>,
        execution: GLExecutionStamp,
    ) -> Self {
        Self {
            base: ManuallyDrop::new(
                nuxie_ore_metal::new_texture_view_backend_base_without_manager(texture, desc),
            ),
            m_glTextureView: 0,
            rust_execution: execution,
        }
    }

    pub(crate) fn executionStamp(&self) -> &GLExecutionStamp {
        &self.rust_execution
    }
}
impl Deref for TextureViewGL {
    type Target = TextureView;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for TextureViewGL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
unsafe impl GpuResourcePayload for TextureViewGL {
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
    fn complete_header_denominator_and_base_layouts_are_frozen() {
        assert_eq!(PINNED_SOURCE.lines().count(), 35);
        assert_eq!(std::mem::offset_of!(TextureGL, base), 0);
        assert_eq!(std::mem::offset_of!(TextureViewGL, base), 0);
        assert!(
            std::mem::offset_of!(TextureGL, rust_execution)
                > std::mem::offset_of!(TextureGL, m_glOwnsTexture)
        );
        assert!(
            std::mem::offset_of!(TextureViewGL, rust_execution)
                > std::mem::offset_of!(TextureViewGL, m_glTextureView)
        );
        assert!(std::mem::size_of::<TextureGL>() > std::mem::size_of::<Texture>() + 16);
    }
}
