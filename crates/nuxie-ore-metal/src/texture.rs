// Mechanical translation of:
// - renderer/include/rive/renderer/ore/ore_texture.hpp
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

#![allow(non_snake_case)]

use crate::gpu_resource::AnyResourceHandle;
use crate::types::{
    TextureAspect, TextureDesc, TextureFormat, TextureType, TextureViewDesc, TextureViewDimension,
};

/// Rust representation of the immutable descriptor snapshot stored by the
/// upstream `Texture` GPU resource.
///
/// C++ has `GPUResource` as a base subobject. Rust represents that base exactly
/// once in `ResourceHandle`; this payload stores only the derived fields.
#[derive(Debug)]
pub struct TextureBase {
    m_width: u32,
    m_height: u32,
    m_depthOrArrayLayers: u32,
    m_format: TextureFormat,
    m_type: TextureType,
    m_renderTarget: bool,
    m_numMipmaps: u32,
    m_sampleCount: u32,
}

impl TextureBase {
    /// Corresponds to `Texture(const TextureDesc&)` and its null manager.
    pub fn new(desc: &TextureDesc<'_>) -> Self {
        Self {
            m_width: desc.width,
            m_height: desc.height,
            m_depthOrArrayLayers: desc.depthOrArrayLayers,
            m_format: desc.format,
            m_type: desc.r#type,
            m_renderTarget: desc.renderTarget,
            m_numMipmaps: desc.numMipmaps,
            m_sampleCount: desc.sampleCount,
        }
    }

    pub fn width(&self) -> u32 {
        self.m_width
    }

    pub fn height(&self) -> u32 {
        self.m_height
    }

    pub fn depthOrArrayLayers(&self) -> u32 {
        self.m_depthOrArrayLayers
    }

    pub fn format(&self) -> TextureFormat {
        self.m_format
    }

    pub fn r#type(&self) -> TextureType {
        self.m_type
    }

    pub fn numMipmaps(&self) -> u32 {
        self.m_numMipmaps
    }

    pub fn sampleCount(&self) -> u32 {
        self.m_sampleCount
    }

    pub fn isRenderTarget(&self) -> bool {
        self.m_renderTarget
    }
}

/// Rust representation of the immutable descriptor and strong source owner
/// stored by the upstream `TextureView` GPU resource.
pub struct TextureViewBase {
    m_texture: AnyResourceHandle,
    m_dimension: TextureViewDimension,
    m_aspect: TextureAspect,
    m_baseMipLevel: u32,
    m_mipCount: u32,
    m_baseLayer: u32,
    m_layerCount: u32,
}

impl TextureViewBase {
    /// Corresponds to `TextureView(rcp<Texture>, const TextureViewDesc&)`.
    pub fn new(desc: &TextureViewDesc<'_>) -> Self {
        Self {
            m_texture: desc.texture.clone(),
            m_dimension: desc.dimension,
            m_aspect: desc.aspect,
            m_baseMipLevel: desc.baseMipLevel,
            m_mipCount: desc.mipCount,
            m_baseLayer: desc.baseLayer,
            m_layerCount: desc.layerCount,
        }
    }

    pub fn texture(&self) -> &AnyResourceHandle {
        &self.m_texture
    }

    pub fn dimension(&self) -> TextureViewDimension {
        self.m_dimension
    }

    pub fn aspect(&self) -> TextureAspect {
        self.m_aspect
    }

    pub fn baseMipLevel(&self) -> u32 {
        self.m_baseMipLevel
    }

    pub fn mipCount(&self) -> u32 {
        self.m_mipCount
    }

    pub fn baseLayer(&self) -> u32 {
        self.m_baseLayer
    }

    pub fn layerCount(&self) -> u32 {
        self.m_layerCount
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_resource::ResourceHandle;
    use crate::types::{BackendId, Texture};
    use std::any::Any;

    struct TestBackend;
    struct TestTexture;

    impl Texture for TestTexture {
        fn backend_id(&self) -> BackendId {
            BackendId::of::<TestBackend>()
        }

        fn as_any(&self) -> &(dyn Any + Send + Sync) {
            self
        }
    }

    #[test]
    fn texture_base_copies_descriptor_fields() {
        let desc = TextureDesc {
            width: 32,
            height: 16,
            depthOrArrayLayers: 4,
            format: TextureFormat::rgba16float,
            r#type: TextureType::texture3D,
            renderTarget: true,
            numMipmaps: 5,
            sampleCount: 4,
            label: Some("texture"),
        };
        let texture = TextureBase::new(&desc);

        assert_eq!(texture.width(), 32);
        assert_eq!(texture.height(), 16);
        assert_eq!(texture.depthOrArrayLayers(), 4);
        assert_eq!(texture.format(), TextureFormat::rgba16float);
        assert_eq!(texture.r#type(), TextureType::texture3D);
        assert!(texture.isRenderTarget());
        assert_eq!(texture.numMipmaps(), 5);
        assert_eq!(texture.sampleCount(), 4);
    }

    #[test]
    fn texture_view_base_retains_source_and_copies_range() {
        let source = ResourceHandle::new(None, TestTexture).erase();
        let desc = TextureViewDesc {
            texture: &source,
            dimension: TextureViewDimension::array2D,
            aspect: TextureAspect::all,
            baseMipLevel: 2,
            mipCount: 3,
            baseLayer: 1,
            layerCount: 4,
        };
        let view = TextureViewBase::new(&desc);

        assert!(view.texture().ptr_eq(&source));
        assert_eq!(source.debugging_ref_count(), 2);
        assert_eq!(view.dimension(), TextureViewDimension::array2D);
        assert_eq!(view.aspect(), TextureAspect::all);
        assert_eq!(view.baseMipLevel(), 2);
        assert_eq!(view.mipCount(), 3);
        assert_eq!(view.baseLayer(), 1);
        assert_eq!(view.layerCount(), 4);
    }
}
