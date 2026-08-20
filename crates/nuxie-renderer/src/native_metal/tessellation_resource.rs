//! Native Metal tessellation texture leaf translated from
//! `renderer/include/rive/renderer/metal/render_context_metal_impl.h:205,252-256`
//! and
//! `renderer/src/metal/render_context_metal_impl.mm:1060-1079`.
//!
//! Pinned upstream source: `rive-runtime` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

use crate::RendererError;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTexture, MTLTextureDescriptor, MTLTextureType,
    MTLTextureUsage,
};

/// `gpu::kTessTextureWidth` from the pinned upstream `gpu.hpp`.
pub(crate) const TESSELLATION_TEXTURE_WIDTH: usize = 2048;

/// `gpu::kTessSpanIndices`, used for the span and reflected span rectangles.
pub(crate) const K_TESS_SPAN_INDICES: [u16; 4 * 3] = [0, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TessellationResourceDescriptor {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) pixel_format: MTLPixelFormat,
    pub(crate) storage_mode: MTLStorageMode,
    pub(crate) texture_type: MTLTextureType,
    pub(crate) mipmap_level_count: usize,
    pub(crate) usage: MTLTextureUsage,
}

impl TessellationResourceDescriptor {
    /// Build the exact private RGBA32Uint 2D descriptor used by upstream.
    /// Zero extents are the clear-resource case; any other width is rejected
    /// because tessellation addressing is defined for 2048 columns.
    pub(crate) fn new(width: usize, height: usize) -> Option<Self> {
        if width == 0 || height == 0 || width != TESSELLATION_TEXTURE_WIDTH {
            return None;
        }
        Some(Self {
            width,
            height,
            pixel_format: MTLPixelFormat::RGBA32Uint,
            storage_mode: MTLStorageMode::Private,
            texture_type: MTLTextureType::Type2D,
            mipmap_level_count: 1,
            usage: MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead,
        })
    }
}

#[derive(Clone)]
pub(crate) struct TessellationResource {
    descriptor: Option<TessellationResourceDescriptor>,
    texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
}

impl TessellationResource {
    pub(crate) fn new(
        device: &ProtocolObject<dyn MTLDevice>,
        width: usize,
        height: usize,
    ) -> Result<Option<Self>, RendererError> {
        let Some(descriptor) = TessellationResourceDescriptor::new(width, height) else {
            return Ok(None);
        };
        let texture = make_texture(device, descriptor)?;
        Ok(Some(Self {
            descriptor: Some(descriptor),
            texture: Some(texture),
        }))
    }

    /// Keep the texture identity when dimensions are unchanged.  Build a
    /// changed-size texture before swapping it in, and clear both owners for
    /// a zero extent.
    pub(crate) fn resize(
        &mut self,
        device: &ProtocolObject<dyn MTLDevice>,
        width: usize,
        height: usize,
    ) -> Result<Option<&ProtocolObject<dyn MTLTexture>>, RendererError> {
        let Some(descriptor) = TessellationResourceDescriptor::new(width, height) else {
            self.descriptor = None;
            self.texture = None;
            return Ok(None);
        };
        if self.descriptor == Some(descriptor) && self.texture.is_some() {
            return Ok(self.texture.as_deref());
        }

        self.resize_with(descriptor, |descriptor| make_texture(device, descriptor))
    }

    /// Replace a resized texture transactionally using the supplied allocator.
    /// The allocator runs before either owner is changed, so an allocation
    /// error leaves the current descriptor and texture identity untouched.
    fn resize_with<F>(
        &mut self,
        descriptor: TessellationResourceDescriptor,
        allocate: F,
    ) -> Result<Option<&ProtocolObject<dyn MTLTexture>>, RendererError>
    where
        F: FnOnce(
            TessellationResourceDescriptor,
        ) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, RendererError>,
    {
        let replacement = allocate(descriptor)?;
        self.descriptor = Some(descriptor);
        self.texture = Some(replacement);
        Ok(self.texture.as_deref())
    }

    pub(crate) fn descriptor(&self) -> Option<TessellationResourceDescriptor> {
        self.descriptor
    }

    pub(crate) fn texture(&self) -> Option<&ProtocolObject<dyn MTLTexture>> {
        self.texture.as_deref()
    }

    pub(crate) fn retained_texture(&self) -> Option<Retained<ProtocolObject<dyn MTLTexture>>> {
        self.texture.clone()
    }
}

fn make_texture(
    device: &ProtocolObject<dyn MTLDevice>,
    descriptor: TessellationResourceDescriptor,
) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, RendererError> {
    let metal_descriptor = MTLTextureDescriptor::new();
    metal_descriptor.setPixelFormat(descriptor.pixel_format);
    // SAFETY: policy dimensions are non-zero and fit losslessly in the
    // platform usize expected by objc2's typed descriptor setters.
    unsafe {
        metal_descriptor.setWidth(descriptor.width);
        metal_descriptor.setHeight(descriptor.height);
        metal_descriptor.setMipmapLevelCount(descriptor.mipmap_level_count);
    }
    metal_descriptor.setStorageMode(descriptor.storage_mode);
    metal_descriptor.setTextureType(descriptor.texture_type);
    metal_descriptor.setUsage(descriptor.usage);
    device
        .newTextureWithDescriptor(&metal_descriptor)
        .ok_or_else(|| {
            RendererError::NativeMetal(
                "failed to allocate tessellation resource texture".to_owned(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_metal::MTLResource;

    #[test]
    fn span_indices_match_upstream_exactly() {
        assert_eq!(K_TESS_SPAN_INDICES, [0, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7]);
    }

    #[test]
    fn descriptor_policy_matches_upstream_tessellation_texture() {
        let descriptor =
            TessellationResourceDescriptor::new(TESSELLATION_TEXTURE_WIDTH, 19).unwrap();
        assert_eq!(descriptor.width, 2048);
        assert_eq!(descriptor.height, 19);
        assert_eq!(descriptor.pixel_format, MTLPixelFormat::RGBA32Uint);
        assert_eq!(descriptor.storage_mode, MTLStorageMode::Private);
        assert_eq!(descriptor.texture_type, MTLTextureType::Type2D);
        assert_eq!(descriptor.mipmap_level_count, 1);
        assert_eq!(
            descriptor.usage,
            MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead
        );
    }

    #[test]
    fn zero_or_noncanonical_extent_has_no_descriptor() {
        for extent in [(0, 1), (TESSELLATION_TEXTURE_WIDTH, 0), (0, 0), (2047, 3)] {
            assert_eq!(
                TessellationResourceDescriptor::new(extent.0, extent.1),
                None
            );
        }
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_tessellation_descriptor_and_resize_preserve_identity() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            return;
        };
        let mut resource = TessellationResource::new(&device, TESSELLATION_TEXTURE_WIDTH, 3)
            .unwrap()
            .unwrap();
        let first = resource.texture().unwrap() as *const ProtocolObject<dyn MTLTexture>;
        let same = resource
            .resize(&device, TESSELLATION_TEXTURE_WIDTH, 3)
            .unwrap()
            .unwrap() as *const ProtocolObject<dyn MTLTexture>;
        assert!(std::ptr::eq(first, same));

        let replacement = resource
            .resize(&device, TESSELLATION_TEXTURE_WIDTH, 4)
            .unwrap()
            .unwrap() as *const ProtocolObject<dyn MTLTexture>;
        assert!(!std::ptr::eq(first, replacement));
        assert_eq!(resource.descriptor().unwrap().height, 4);
        let texture = resource.texture().unwrap();
        assert_eq!(texture.width(), 2048);
        assert_eq!(texture.height(), 4);
        assert_eq!(texture.pixelFormat(), MTLPixelFormat::RGBA32Uint);
        assert_eq!(texture.storageMode(), MTLStorageMode::Private);
        assert_eq!(texture.textureType(), MTLTextureType::Type2D);
        assert_eq!(texture.mipmapLevelCount(), 1);
        assert_eq!(
            texture.usage(),
            MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead
        );

        assert!(resource
            .resize(&device, TESSELLATION_TEXTURE_WIDTH, 0)
            .unwrap()
            .is_none());
        assert!(resource.texture().is_none());
        assert!(resource.descriptor().is_none());
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn failed_tessellation_resize_preserves_identity_and_real_retry_succeeds() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            return;
        };
        let mut resource = TessellationResource::new(&device, TESSELLATION_TEXTURE_WIDTH, 3)
            .unwrap()
            .unwrap();
        let clone = resource.clone();
        let before = resource.texture().unwrap() as *const ProtocolObject<dyn MTLTexture>;
        assert!(std::ptr::eq(
            before,
            clone.texture().unwrap() as *const ProtocolObject<dyn MTLTexture>
        ));
        let before_descriptor = resource.descriptor();
        let replacement =
            TessellationResourceDescriptor::new(TESSELLATION_TEXTURE_WIDTH, 4).unwrap();

        let failed = resource.resize_with(replacement, |_| {
            Err(RendererError::NativeMetal(
                "injected allocation failure".to_owned(),
            ))
        });
        assert!(failed.is_err());
        assert_eq!(resource.descriptor(), before_descriptor);
        assert!(std::ptr::eq(
            before,
            resource.texture().unwrap() as *const ProtocolObject<dyn MTLTexture>
        ));

        let retained_before_retry = resource.retained_texture().unwrap();
        let after = resource
            .resize(&device, TESSELLATION_TEXTURE_WIDTH, 4)
            .unwrap()
            .unwrap() as *const ProtocolObject<dyn MTLTexture>;
        assert!(!std::ptr::eq(before, after));
        assert_eq!(retained_before_retry.width(), 2048);
        assert_eq!(resource.descriptor().unwrap().height, 4);
    }
}
