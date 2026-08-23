// Mechanical translation of:
// - renderer/src/ore/metal/ore_texture_metal.hpp
// - renderer/src/ore/metal/ore_texture_metal.mm
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

#![allow(non_snake_case)]

use std::any::Any;

use crate::gpu_resource::{GpuResourceManager, ResourceHandle};
use crate::texture::{TextureBase, TextureViewBase};
use crate::types::{BackendId, Texture, TextureDesc, TextureView, TextureViewDesc};
#[cfg(any(test, target_vendor = "apple"))]
use crate::types::{TextureDataDesc, TextureType};

use super::MetalBackend;

#[cfg(target_vendor = "apple")]
use std::ffi::c_void;
#[cfg(target_vendor = "apple")]
use std::ptr::NonNull;

#[cfg(target_vendor = "apple")]
use objc2::rc::Retained;
#[cfg(target_vendor = "apple")]
use objc2::runtime::ProtocolObject;
#[cfg(target_vendor = "apple")]
use objc2_metal::{MTLOrigin, MTLRegion, MTLSize, MTLTexture};

#[cfg(target_vendor = "apple")]
struct RetainedMetalTexture(Retained<ProtocolObject<dyn MTLTexture>>);

// SAFETY: Metal texture objects support concurrent retain/release and API
// access. Synchronization of CPU/GPU contents is still enforced at each
// unsafe upload/encoder call; this wrapper only makes the upstream thread-safe
// intrusive ownership contract explicit to Rust.
#[cfg(target_vendor = "apple")]
unsafe impl Send for RetainedMetalTexture {}
// SAFETY: Same invariant as the `Send` implementation above.
#[cfg(target_vendor = "apple")]
unsafe impl Sync for RetainedMetalTexture {}

/// Pure upload policy shared by portable tests and the native `replaceRegion`
/// call. This is a value calculation, not a deferred GPU command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(any(test, target_vendor = "apple"))]
struct TextureUploadLayout {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mipmap_level: u32,
    pub slice: u32,
    pub bytes_per_row: u32,
    pub bytes_per_image: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureUploadError {
    MissingNativeTexture,
    EmptyData,
    SizeOverflow,
    DataTooShort { required: usize, actual: usize },
}

#[cfg(any(test, target_vendor = "apple"))]
fn validate_upload_span(
    texture_type: TextureType,
    data: &TextureDataDesc<'_>,
    layout: TextureUploadLayout,
) -> Result<(), TextureUploadError> {
    if data.data.is_empty() {
        return Err(TextureUploadError::EmptyData);
    }
    let row_bytes =
        usize::try_from(data.bytesPerRow).map_err(|_| TextureUploadError::SizeOverflow)?;
    let required = match texture_type {
        TextureType::texture3D => layout
            .bytes_per_image
            .checked_mul(data.depth as usize)
            .ok_or(TextureUploadError::SizeOverflow)?,
        TextureType::array2D => layout.bytes_per_image,
        TextureType::texture2D | TextureType::cube => row_bytes
            .checked_mul(data.height as usize)
            .ok_or(TextureUploadError::SizeOverflow)?,
    };
    if data.data.len() < required {
        return Err(TextureUploadError::DataTooShort {
            required,
            actual: data.data.len(),
        });
    }
    Ok(())
}

#[cfg(any(test, target_vendor = "apple"))]
fn upload_layout(
    texture_type: TextureType,
    data: &TextureDataDesc<'_>,
) -> Result<TextureUploadLayout, TextureUploadError> {
    let bytes_per_image = if matches!(texture_type, TextureType::texture3D | TextureType::array2D) {
        usize::try_from(data.bytesPerRow)
            .ok()
            .and_then(|row| {
                usize::try_from(data.rowsPerImage)
                    .ok()
                    .and_then(|rows| row.checked_mul(rows))
            })
            .ok_or(TextureUploadError::SizeOverflow)?
    } else {
        0
    };

    Ok(TextureUploadLayout {
        x: data.x,
        y: data.y,
        z: data.z,
        width: data.width,
        height: data.height,
        depth: data.depth,
        mipmap_level: data.mipLevel,
        slice: data.layer,
        bytes_per_row: data.bytesPerRow,
        bytes_per_image,
    })
}

/// Concrete Metal texture corresponding to `TextureMetal`.
pub struct TextureMetal {
    base: TextureBase,
    #[cfg(target_vendor = "apple")]
    m_mtlTexture: Option<RetainedMetalTexture>,
}

impl TextureMetal {
    /// Corresponds to `TextureMetal(const TextureDesc&)`; native publication
    /// remains nil until ContextMetal allocates the texture successfully.
    pub fn new(desc: &TextureDesc<'_>) -> Self {
        Self {
            base: TextureBase::new(desc),
            #[cfg(target_vendor = "apple")]
            m_mtlTexture: None,
        }
    }

    #[cfg(target_vendor = "apple")]
    pub fn with_native_texture(
        desc: &TextureDesc<'_>,
        native_texture: Retained<ProtocolObject<dyn MTLTexture>>,
    ) -> Self {
        Self {
            base: TextureBase::new(desc),
            m_mtlTexture: Some(RetainedMetalTexture(native_texture)),
        }
    }

    pub fn into_resource(self, manager: Option<GpuResourceManager>) -> ResourceHandle<Self> {
        ResourceHandle::new(manager, self)
    }

    pub fn base(&self) -> &TextureBase {
        &self.base
    }

    #[cfg(target_vendor = "apple")]
    pub fn mtlTexture(&self) -> Option<&ProtocolObject<dyn MTLTexture>> {
        self.m_mtlTexture.as_ref().map(|texture| texture.0.as_ref())
    }

    /// Invoke the exact Objective-C++ `replaceRegion` upload on Apple.
    #[cfg(target_vendor = "apple")]
    pub fn upload(&self, data: &TextureDataDesc<'_>) -> Result<(), TextureUploadError> {
        let texture: &ProtocolObject<dyn MTLTexture> = self
            .m_mtlTexture
            .as_ref()
            .map(|texture| texture.0.as_ref())
            .ok_or(TextureUploadError::MissingNativeTexture)?;
        let layout = upload_layout(self.base.r#type(), data)?;
        validate_upload_span(self.base.r#type(), data, layout)?;
        let pointer = NonNull::new(data.data.as_ptr().cast_mut().cast::<c_void>())
            .ok_or(TextureUploadError::EmptyData)?;
        let region = MTLRegion {
            origin: MTLOrigin {
                x: layout.x as usize,
                y: layout.y as usize,
                z: layout.z as usize,
            },
            size: MTLSize {
                width: layout.width as usize,
                height: layout.height as usize,
                depth: layout.depth as usize,
            },
        };
        // SAFETY: `data.data` remains live for the synchronous Metal upload;
        // the caller-supplied row/image strides and region are forwarded
        // unchanged, and all u32 coordinates widen losslessly to NSUInteger.
        unsafe {
            texture.replaceRegion_mipmapLevel_slice_withBytes_bytesPerRow_bytesPerImage(
                region,
                layout.mipmap_level as usize,
                layout.slice as usize,
                pointer,
                layout.bytes_per_row as usize,
                layout.bytes_per_image,
            );
        }
        Ok(())
    }
}

impl Texture for TextureMetal {
    fn backend_id(&self) -> BackendId {
        BackendId::of::<MetalBackend>()
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

/// Concrete Metal texture view corresponding to `TextureViewMetal`.
pub struct TextureViewMetal {
    base: TextureViewBase,
    #[cfg(target_vendor = "apple")]
    m_mtlTextureView: Option<RetainedMetalTexture>,
}

impl TextureViewMetal {
    pub fn new(desc: &TextureViewDesc<'_>) -> Self {
        Self {
            base: TextureViewBase::new(desc),
            #[cfg(target_vendor = "apple")]
            m_mtlTextureView: None,
        }
    }

    #[cfg(target_vendor = "apple")]
    pub fn with_native_texture_view(
        desc: &TextureViewDesc<'_>,
        native_view: Retained<ProtocolObject<dyn MTLTexture>>,
    ) -> Self {
        Self {
            base: TextureViewBase::new(desc),
            m_mtlTextureView: Some(RetainedMetalTexture(native_view)),
        }
    }

    pub fn base(&self) -> &TextureViewBase {
        &self.base
    }

    pub fn into_resource(self, manager: Option<GpuResourceManager>) -> ResourceHandle<Self> {
        ResourceHandle::new(manager, self)
    }

    /// Return the view, falling back to the retained source texture exactly
    /// as the upstream conditional expression does.
    #[cfg(target_vendor = "apple")]
    pub fn mtlTexture(&self) -> Option<&ProtocolObject<dyn MTLTexture>> {
        self.m_mtlTextureView
            .as_ref()
            .map(|texture| texture.0.as_ref())
            .or_else(|| {
                self.base
                    .texture()
                    .downcast_ref::<TextureMetal>()
                    .and_then(TextureMetal::mtlTexture)
            })
    }
}

impl TextureView for TextureViewMetal {
    fn backend_id(&self) -> BackendId {
        BackendId::of::<MetalBackend>()
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

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
            data,
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
        let plain = upload_layout(TextureType::texture2D, &data).expect("2D upload layout");
        assert_eq!((plain.x, plain.y, plain.z), (1, 2, 3));
        assert_eq!((plain.width, plain.height, plain.depth), (4, 5, 6));
        assert_eq!((plain.mipmap_level, plain.slice), (2, 3));
        assert_eq!(plain.bytes_per_row, 16);
        assert_eq!(plain.bytes_per_image, 0);

        for texture_type in [TextureType::texture3D, TextureType::array2D] {
            assert_eq!(
                upload_layout(texture_type, &data)
                    .expect("layered upload layout")
                    .bytes_per_image,
                64
            );
        }
    }

    #[test]
    fn safe_upload_span_validation_rejects_metal_overreads() {
        let short = upload_desc(&[0; 8]);
        let plain = upload_layout(TextureType::texture2D, &short).expect("2D upload layout");
        assert_eq!(
            validate_upload_span(TextureType::texture2D, &short, plain),
            Err(TextureUploadError::DataTooShort {
                required: 80,
                actual: 8,
            })
        );
    }

    #[test]
    fn texture_view_retains_the_exact_intrusive_source() {
        let source = TextureMetal::new(&texture_desc(TextureType::texture2D))
            .into_resource(None)
            .erase();
        let desc = TextureViewDesc {
            texture: &source,
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        };
        let view = TextureViewMetal::new(&desc);
        assert!(view.base().texture().ptr_eq(&source));
        assert_eq!(source.debugging_ref_count(), 2);
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
        let texture = TextureMetal::with_native_texture(&desc, native);
        texture
            .upload(&TextureDataDesc {
                data: &[1, 2, 3, 4],
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

        let source = texture.into_resource(None).erase();
        let view_desc = TextureViewDesc {
            texture: &source,
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        };
        let view = TextureViewMetal::new(&view_desc);
        assert!(view.mtlTexture().is_some());
    }
}
