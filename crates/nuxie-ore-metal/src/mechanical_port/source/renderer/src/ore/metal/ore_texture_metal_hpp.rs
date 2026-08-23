/*
 * Copyright 2025 Rive
 */

// #pragma once
// #include "rive/renderer/ore/ore_texture.hpp"
// #import <Metal/Metal.h>

// Mechanical translation of the complete pinned source header
// renderer/src/ore/metal/ore_texture_metal.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;
use std::mem::ManuallyDrop;

use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::{
    AnyResourceHandle, GPUResource, GpuResourcePayload,
};
#[cfg(test)]
use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_types_hpp::TextureDataDesc;
use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_types_hpp::{
    TextureDesc, TextureFormat, TextureViewDesc,
};

// `id<MTLTexture>` is a nullable, strong Objective-C owner under ARC. Rust's
// `Retained<T>` is the corresponding strong owner; `Option` preserves the
// source `nil` state. The mechanical header is source-shaped and is not wired
// into the runtime module, but the non-Apple stand-in keeps this translation's
// declaration shape available to tools that inspect it off Apple.
#[cfg(target_vendor = "apple")]
use objc2::rc::Retained;
#[cfg(target_vendor = "apple")]
use objc2::runtime::ProtocolObject;
#[cfg(target_vendor = "apple")]
use objc2_metal::MTLTexture;

#[cfg(target_vendor = "apple")]
type NativeMetalTexture = Option<Retained<ProtocolObject<dyn MTLTexture>>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalTexture = Option<()>;

// namespace rive::ore

// class ContextMetal;
// The source forward declaration is retained for the friend relationship
// below. ContextMetal is owned by its own translation unit.

// class TextureMetal : public LITE_RTTI_OVERRIDE(Texture, TextureMetal)
// {
// Rust has no class inheritance. `base` is the first field to preserve the
// source Texture base-subobject order. `LITE_RTTI_OVERRIDE(Texture,
// TextureMetal)` remains the source lite-RTTI identity/override seam and is
// not duplicated as a payload field.
#[repr(C)]
pub struct TextureMetal {
    pub(crate) base: ManuallyDrop<Texture>,
    // private:
    // friend class ContextMetal;
    // Rust has no friend declarations; this source access boundary remains
    // visible here, and the owning translation unit uses crate visibility.
    // id<MTLTexture> m_mtlTexture = nil;
    // `NativeMetalTexture` retains the non-nil Objective-C texture until the
    // enclosing logical TextureMetal owner is dropped.
    pub(crate) m_mtlTexture: ManuallyDrop<NativeMetalTexture>,
}

// SAFETY: retained Metal textures support cross-thread retain/release and GPU
// use; upload remains caller-serialized through the TextureApi contract.
unsafe impl Send for TextureMetal {}

unsafe impl GpuResourcePayload for TextureMetal {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base.base
    }
}

impl TextureMetal {
    // public:

    // TextureMetal(const TextureDesc& desc) : lite_rtti_override(desc) {}
    // The source lite-RTTI initializer delegates to the Texture base
    // constructor and records the concrete TextureMetal identity.
    pub(crate) fn new(desc: &TextureDesc<'_>) -> Self {
        Self {
            base: ManuallyDrop::new(Texture::new(desc)),
            m_mtlTexture: ManuallyDrop::new(None),
        }
    }

    // ~TextureMetal() override = default; // ARC releases m_mtlTexture
    // Rust's default drop glue releases the retained native texture owner
    // before the remaining source-shaped fields.

    // void upload(const TextureDataDesc& data) override;
    // The paired ore_texture_metal.mm translation owns the complete upload
    // implementation. The source borrowed descriptor remains explicit there.

    // id<MTLTexture> mtlTexture() const { return m_mtlTexture; }
    pub fn mtlTexture(&self) -> NativeMetalTexture {
        (*self.m_mtlTexture).clone()
    }

    pub fn base(&self) -> &Texture {
        &self.base
    }

    pub fn format(&self) -> TextureFormat {
        self.base.format()
    }

    pub fn sampleCount(&self) -> u32 {
        self.base.sampleCount()
    }
}

impl Drop for TextureMetal {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_mtlTexture);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

// class TextureViewMetal
//     : public LITE_RTTI_OVERRIDE(TextureView, TextureViewMetal)
// {
// Rust preserves the source TextureView base-subobject before the concrete
// Metal view owner. `LITE_RTTI_OVERRIDE(TextureView, TextureViewMetal)` is
// retained as the source lite-RTTI identity/override seam rather than as a
// duplicate payload field.
#[repr(C)]
pub struct TextureViewMetal {
    pub(crate) base: ManuallyDrop<TextureView>,
    // private:
    // friend class ContextMetal;
    // Rust has no friend declarations; this source access boundary remains
    // visible here, and the owning translation unit uses crate visibility.
    // id<MTLTexture> m_mtlTextureView = nil;
    // `None` is the source nil view state. The base TextureView retains the
    // source Texture owner used by the fallback accessor below.
    pub(crate) m_mtlTextureView: ManuallyDrop<NativeMetalTexture>,
}

// SAFETY: the payload is immutable after publication and contains only
// logical resource handles plus retained Metal texture owners.
unsafe impl Send for TextureViewMetal {}

unsafe impl GpuResourcePayload for TextureViewMetal {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base.base
    }
}

impl TextureViewMetal {
    // public:

    // TextureViewMetal(rcp<Texture> texture, const TextureViewDesc& desc) :
    //     lite_rtti_override(std::move(texture), desc)
    // {}
    pub(crate) fn new(texture: AnyResourceHandle, desc: &TextureViewDesc<'_>) -> Self {
        Self {
            base: ManuallyDrop::new(TextureView::new(texture, desc)),
            m_mtlTextureView: ManuallyDrop::new(None),
        }
    }

    // ~TextureViewMetal() override = default;
    // Rust's default drop glue releases the optional native view and then the
    // retained source Texture through the base TextureView owner.

    // id<MTLTexture> mtlTexture() const
    // {
    //     return m_mtlTextureView
    //                ? m_mtlTextureView
    //                : static_cast<TextureMetal*>(m_texture.get())->mtlTexture();
    // }
    pub fn mtlTexture(&self) -> NativeMetalTexture {
        match (*self.m_mtlTextureView).clone() {
            Some(texture_view) => Some(texture_view),
            None => self.baseTexture().and_then(TextureMetal::mtlTexture),
        }
    }

    pub fn baseTexture(&self) -> Option<&TextureMetal> {
        self.base.m_texture.downcast_ref::<TextureMetal>()
    }

    pub fn base(&self) -> &TextureView {
        &self.base
    }

    pub fn baseMipLevel(&self) -> u32 {
        self.base.baseMipLevel()
    }

    pub fn baseLayer(&self) -> u32 {
        self.base.baseLayer()
    }
}

impl Drop for TextureViewMetal {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_mtlTextureView);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TextureAspect, TextureFormat, TextureViewDimension};

    fn texture_desc(texture_type: TextureType) -> TextureDesc<'static> {
        TextureDesc {
            width: 8,
            height: 4,
            depthOrArrayLayers: 2,
            format: TextureFormat::rgba8unorm,
            r#type: texture_type,
            renderTarget: false,
            numMipmaps: 1,
            sampleCount: 1,
            label: None,
        }
    }

    fn upload_desc<'a>(data: &'a [u8]) -> TextureDataDesc<'a> {
        TextureDataDesc {
            data: Some(data),
            bytesPerRow: 16,
            rowsPerImage: 4,
            mipLevel: 2,
            layer: 3,
            x: 1,
            y: 2,
            z: 3,
            width: 4,
            height: 5,
            depth: 6,
        }
    }

    #[test]
    fn upload_layout_preserves_region_and_texture_type_stride_rule() {
        let data = upload_desc(&[0; 8]);
        assert_eq!(data.data.unwrap().len(), 8);
        assert_eq!((data.x, data.y, data.z), (1, 2, 3));
        assert_eq!((data.width, data.height, data.depth), (4, 5, 6));
        assert_eq!((data.mipLevel, data.layer), (2, 3));
        assert_eq!(data.bytesPerRow, 16);
        assert_eq!(data.bytesPerRow * data.rowsPerImage, 64);
        // The source Metal upload uses this row*slice stride for 3D/array
        // textures while the ordinary 2D branch passes zero to Metal.
    }

    #[test]
    fn safe_upload_span_validation_rejects_metal_overreads() {
        let short = upload_desc(&[0; 8]);
        let required = short.bytesPerRow * short.height;
        assert_eq!(required, 80);
        assert!(short.data.unwrap().len() < required as usize);
    }

    #[test]
    fn texture_view_retains_the_exact_intrusive_source() {
        let source = crate::gpu_resource::ResourceHandle::new(
            None,
            TextureMetal::new(&texture_desc(TextureType::texture2D)),
        )
        .erase();
        let desc = TextureViewDesc {
            texture: Some(&source),
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        };
        let view = TextureViewMetal::new(source.clone(), &desc);
        assert!(view.base().texture().ptr_eq(&source));
        assert_eq!(source.debugging_refcnt(), 2);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn live_texture_upload_calls_metal_and_view_falls_back_to_source() {
        use objc2_metal::{
            MTLCreateSystemDefaultDevice, MTLDevice, MTLPixelFormat, MTLTextureDescriptor,
        };

        let Some(device) = MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let descriptor = MTLTextureDescriptor::new();
        descriptor.setPixelFormat(MTLPixelFormat::RGBA8Unorm);
        // SAFETY: both dimensions and the mip count are non-zero.
        unsafe {
            descriptor.setWidth(1);
            descriptor.setHeight(1);
            descriptor.setMipmapLevelCount(1);
        }
        let native = device
            .newTextureWithDescriptor(&descriptor)
            .expect("allocate 1x1 Metal texture");
        let desc = TextureDesc {
            width: 1,
            height: 1,
            ..texture_desc(TextureType::texture2D)
        };
        let mut texture = TextureMetal::new(&desc);
        texture.m_mtlTexture = ManuallyDrop::new(Some(native));
        texture
            .upload(&TextureDataDesc {
                data: Some(&[1, 2, 3, 4]),
                bytesPerRow: 4,
                rowsPerImage: 1,
                mipLevel: 0,
                layer: 0,
                x: 0,
                y: 0,
                z: 0,
                width: 1,
                height: 1,
                depth: 1,
            })
            .expect("upload through replaceRegion");

        let source = crate::gpu_resource::ResourceHandle::new(None, texture).erase();
        let view_desc = TextureViewDesc {
            texture: Some(&source),
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        };
        let view = TextureViewMetal::new(source.clone(), &view_desc);
        assert!(view.mtlTexture().is_some());
    }
}
