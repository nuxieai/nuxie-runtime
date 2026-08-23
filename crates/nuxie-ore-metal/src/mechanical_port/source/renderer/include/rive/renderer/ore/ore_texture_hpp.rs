/*
 * Copyright 2025 Rive
 */

// #pragma once

// #include "rive/renderer/gpu_resource.hpp"
// #include "utils/lite_rtti.hpp"
// #include "rive/renderer/ore/ore_types.hpp"

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/ore/ore_texture.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

use super::super::gpu_resource_hpp::{AnyResourceHandle, GPUResource, GpuResourcePayload};
use super::ore_types_hpp::{
    TextureAspect, TextureDataDesc, TextureDesc, TextureFormat, TextureType, TextureViewDesc,
    TextureViewDimension,
};

// namespace rive::ore
// {

pub trait TextureApi {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn depthOrArrayLayers(&self) -> u32;
    fn format(&self) -> TextureFormat;
    fn r#type(&self) -> TextureType;
    fn numMipmaps(&self) -> u32;
    fn sampleCount(&self) -> u32;
    fn isRenderTarget(&self) -> bool;
    fn upload(&self, data: &TextureDataDesc<'_>) -> Result<(), TextureUploadError>;

    /// Concrete-backend ownership adaptation for source implementations that
    /// call `ref_rcp(this)` during upload. The type-erased call site supplies
    /// exactly one retained handle; backends that do not queue the texture use
    /// the source-default upload path and release it on return.
    fn uploadWithOwner(
        &self,
        data: &TextureDataDesc<'_>,
        _owner: AnyResourceHandle,
    ) -> Result<(), TextureUploadError> {
        self.upload(data)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureUploadError {
    WrongResourceKind,
    WrongExecutionDomain,
    MissingNativeTexture,
    NullData,
    EmptyData,
    SizeOverflow,
    DataTooShort { required: usize, actual: usize },
}

// class Context;
// The source forward declaration is retained for the friend relationships
// below. Context is owned by its own translation unit.

// class Texture : public rive::gpu::GPUResource, public ENABLE_LITE_RTTI(Texture)
// class Texture : public rive::gpu::GPUResource,
//                 public ENABLE_LITE_RTTI(Texture)
// {
//
// The complete concrete payload is stored in `ResourceAllocation<T>`, which
// owns the source GPUResource base state once and dispatches concrete Drop.
#[repr(C)]
pub struct TextureMembers {
    // uint32_t m_width;
    pub(crate) m_width: u32,
    // uint32_t m_height;
    pub(crate) m_height: u32,
    // uint32_t m_depthOrArrayLayers;
    pub(crate) m_depthOrArrayLayers: u32,
    // TextureFormat m_format;
    pub(crate) m_format: TextureFormat,
    // TextureType m_type;
    pub(crate) m_type: TextureType,
    // bool m_renderTarget;
    pub(crate) m_renderTarget: bool,
    // uint32_t m_numMipmaps;
    pub(crate) m_numMipmaps: u32,
    // uint32_t m_sampleCount;
    pub(crate) m_sampleCount: u32,
    // };
}

#[repr(C)]
pub struct Texture {
    pub(crate) base: ManuallyDrop<GPUResource>,
    pub(crate) members: ManuallyDrop<TextureMembers>,
}

impl Deref for Texture {
    type Target = TextureMembers;

    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl DerefMut for Texture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.members
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        #[cfg(test)]
        super::super::gpu_resource_hpp::record_resource_drop_stage("Texture.base");
        unsafe { ManuallyDrop::drop(&mut self.base) };
    }
}

unsafe impl GpuResourcePayload for Texture {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base
    }
}

impl Texture {
    // public:

    // uint32_t width() const { return m_width; }
    pub fn width(&self) -> u32 {
        self.m_width
    }

    // uint32_t height() const { return m_height; }
    pub fn height(&self) -> u32 {
        self.m_height
    }

    // uint32_t depthOrArrayLayers() const { return m_depthOrArrayLayers; }
    pub fn depthOrArrayLayers(&self) -> u32 {
        self.m_depthOrArrayLayers
    }

    // TextureFormat format() const { return m_format; }
    pub fn format(&self) -> TextureFormat {
        self.m_format
    }

    // TextureType type() const { return m_type; }
    // `type` is a Rust keyword, so the raw identifier preserves the source
    // accessor spelling at the call site: `texture.r#type()`.
    pub fn r#type(&self) -> TextureType {
        self.m_type
    }

    // uint32_t numMipmaps() const { return m_numMipmaps; }
    pub fn numMipmaps(&self) -> u32 {
        self.m_numMipmaps
    }

    // uint32_t sampleCount() const { return m_sampleCount; }
    pub fn sampleCount(&self) -> u32 {
        self.m_sampleCount
    }

    // bool isRenderTarget() const { return m_renderTarget; }
    pub fn isRenderTarget(&self) -> bool {
        self.m_renderTarget
    }

    // virtual void upload(const TextureDataDesc& data) = 0;
    // `TextureApi` above retains this pure-virtual dispatch surface.

    // virtual ~Texture() = default;
    // Rust's default drop glue supplies the virtual-destructor boundary for
    // the concrete resource owner; no extra state is introduced here.

    // protected:
    // friend class Context;
    // friend class TextureView;
    // friend class RenderPass;
    // Rust has no friend declarations; these source access boundaries remain
    // visible above, and the owning translation units use crate visibility.

    // Texture(const TextureDesc& desc) :
    //     rive::gpu::GPUResource(nullptr),
    //     m_width(desc.width),
    //     m_height(desc.height),
    //     m_depthOrArrayLayers(desc.depthOrArrayLayers),
    //     m_format(desc.format),
    //     m_type(desc.type),
    //     m_renderTarget(desc.renderTarget),
    //     m_numMipmaps(desc.numMipmaps),
    //     m_sampleCount(desc.sampleCount)
    // {}
    pub(crate) fn new(desc: &TextureDesc<'_>) -> Self {
        Self {
            base: ManuallyDrop::new(GPUResource::new(None)),
            members: ManuallyDrop::new(TextureMembers {
                m_width: desc.width,
                m_height: desc.height,
                m_depthOrArrayLayers: desc.depthOrArrayLayers,
                m_format: desc.format,
                m_type: desc.r#type,
                m_renderTarget: desc.renderTarget,
                m_numMipmaps: desc.numMipmaps,
                m_sampleCount: desc.sampleCount,
            }),
        }
    }

    // Texture(rcp<rive::gpu::GPUResourceManager> manager,
    //         const TextureDesc& desc) :
    //     rive::gpu::GPUResource(std::move(manager)),
    //     m_width(desc.width),
    //     m_height(desc.height),
    //     m_depthOrArrayLayers(desc.depthOrArrayLayers),
    //     m_format(desc.format),
    //     m_type(desc.type),
    //     m_renderTarget(desc.renderTarget),
    //     m_numMipmaps(desc.numMipmaps),
    //     m_sampleCount(desc.sampleCount)
    // {}
    // The manager-taking constructor is represented at concrete publication:
    // `ResourceHandle::new(Some(manager), TextureMetal { ... })`.
}

// Source member declarations retained in their pinned order after the
// constructor declarations above:
// uint32_t m_width;
// uint32_t m_height;
// uint32_t m_depthOrArrayLayers;
// TextureFormat m_format;
// TextureType m_type;
// bool m_renderTarget;
// uint32_t m_numMipmaps;
// uint32_t m_sampleCount;
// };

// class TextureView : public rive::gpu::GPUResource,
//                     public ENABLE_LITE_RTTI(TextureView)
// {
//
// Rust preserves the source inheritance order in `base`. The lite-RTTI base
// remains a source-visible contract rather than a duplicate payload field.
#[repr(C)]
pub struct TextureViewMembers {
    // Strong source owner: the view retains its texture for the entire view
    // lifetime, matching `rcp<Texture> m_texture` in the pinned header.
    // rcp<Texture> m_texture;
    pub(crate) m_texture: AnyResourceHandle,
    // TextureViewDimension m_dimension;
    pub(crate) m_dimension: TextureViewDimension,
    // TextureAspect m_aspect;
    pub(crate) m_aspect: TextureAspect,
    // uint32_t m_baseMipLevel;
    pub(crate) m_baseMipLevel: u32,
    // uint32_t m_mipCount;
    pub(crate) m_mipCount: u32,
    // uint32_t m_baseLayer;
    pub(crate) m_baseLayer: u32,
    // uint32_t m_layerCount;
    pub(crate) m_layerCount: u32,
    // };
}

#[repr(C)]
pub struct TextureView {
    pub(crate) base: ManuallyDrop<GPUResource>,
    pub(crate) members: ManuallyDrop<TextureViewMembers>,
}

impl Deref for TextureView {
    type Target = TextureViewMembers;

    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl DerefMut for TextureView {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.members
    }
}

impl Drop for TextureView {
    fn drop(&mut self) {
        unsafe {
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("TextureView.texture");
            core::ptr::drop_in_place(&mut self.m_texture);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("TextureView.base");
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

unsafe impl GpuResourcePayload for TextureView {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base
    }
}

impl TextureView {
    // public:

    // Texture* texture() const { return m_texture.get(); }
    // The nullable raw-pointer result is represented as an optional borrow;
    // `m_texture` itself remains the owning `rcp<Texture>`.
    pub fn texture(&self) -> &AnyResourceHandle {
        &self.m_texture
    }

    // TextureViewDimension dimension() const { return m_dimension; }
    pub fn dimension(&self) -> TextureViewDimension {
        self.m_dimension
    }

    // TextureAspect aspect() const { return m_aspect; }
    pub fn aspect(&self) -> TextureAspect {
        self.m_aspect
    }

    // uint32_t baseMipLevel() const { return m_baseMipLevel; }
    pub fn baseMipLevel(&self) -> u32 {
        self.m_baseMipLevel
    }

    // uint32_t mipCount() const { return m_mipCount; }
    pub fn mipCount(&self) -> u32 {
        self.m_mipCount
    }

    // uint32_t baseLayer() const { return m_baseLayer; }
    pub fn baseLayer(&self) -> u32 {
        self.m_baseLayer
    }

    // uint32_t layerCount() const { return m_layerCount; }
    pub fn layerCount(&self) -> u32 {
        self.m_layerCount
    }

    // virtual ~TextureView() = default;

    // protected:
    // friend class Context;
    // friend class RenderPass;
    // Rust has no friend declarations; these source access boundaries remain
    // visible here, and the owning translation units use crate visibility.

    // TextureView(rcp<Texture> texture, const TextureViewDesc& desc) :
    //     rive::gpu::GPUResource(nullptr),
    //     m_texture(std::move(texture)),
    //     m_dimension(desc.dimension),
    //     m_aspect(desc.aspect),
    //     m_baseMipLevel(desc.baseMipLevel),
    //     m_mipCount(desc.mipCount),
    //     m_baseLayer(desc.baseLayer),
    //     m_layerCount(desc.layerCount)
    // {}
    pub(crate) fn new(texture: AnyResourceHandle, desc: &TextureViewDesc<'_>) -> Self {
        Self {
            base: ManuallyDrop::new(GPUResource::new(None)),
            members: ManuallyDrop::new(TextureViewMembers {
                m_texture: texture,
                m_dimension: desc.dimension,
                m_aspect: desc.aspect,
                m_baseMipLevel: desc.baseMipLevel,
                m_mipCount: desc.mipCount,
                m_baseLayer: desc.baseLayer,
                m_layerCount: desc.layerCount,
            }),
        }
    }

    // TextureView(rcp<rive::gpu::GPUResourceManager> manager,
    //             rcp<Texture> texture,
    //             const TextureViewDesc& desc) :
    //     rive::gpu::GPUResource(std::move(manager)),
    //     m_texture(std::move(texture)),
    //     m_dimension(desc.dimension),
    //     m_aspect(desc.aspect),
    //     m_baseMipLevel(desc.baseMipLevel),
    //     m_mipCount(desc.mipCount),
    //     m_baseLayer(desc.baseLayer),
    //     m_layerCount(desc.layerCount)
    // {}
    // Manager ownership is supplied by the outer concrete resource handle.
}

// Source member declarations retained in their pinned order after the
// constructor declarations above:
// rcp<Texture> m_texture;
// TextureViewDimension m_dimension;
// TextureAspect m_aspect;
// uint32_t m_baseMipLevel;
// uint32_t m_mipCount;
// uint32_t m_baseLayer;
// uint32_t m_layerCount;
// };

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::ResourceHandle;

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
        let texture = Texture::new(&desc);
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
        let desc = TextureDesc {
            width: 4,
            height: 4,
            ..TextureDesc::default()
        };
        let source = ResourceHandle::new(None, Texture::new(&desc)).erase();
        let view_desc = TextureViewDesc {
            texture: Some(&source),
            dimension: TextureViewDimension::array2D,
            aspect: TextureAspect::all,
            baseMipLevel: 2,
            mipCount: 3,
            baseLayer: 1,
            layerCount: 4,
        };
        let view = TextureView::new(source.clone(), &view_desc);
        assert!(view.texture().ptr_eq(&source));
        assert_eq!(source.debugging_refcnt(), 2);
        assert_eq!(view.dimension(), TextureViewDimension::array2D);
        assert_eq!(view.aspect(), TextureAspect::all);
        assert_eq!(view.baseMipLevel(), 2);
        assert_eq!(view.mipCount(), 3);
        assert_eq!(view.baseLayer(), 1);
        assert_eq!(view.layerCount(), 4);
    }
}
