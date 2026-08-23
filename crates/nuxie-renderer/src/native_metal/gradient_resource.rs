//! Native Metal gradient texture leaf translated from
//! `renderer/include/rive/renderer/metal/render_context_metal_impl.h:204,244-247`
//! and
//! `renderer/src/metal/render_context_metal_impl.mm:1041-1058`.
//!
//! Pinned upstream source: `rive-runtime` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
//!
//! Upstream callers always request `gpu::kGradTextureWidth` (512) and vary
//! only the resource height.  The descriptor policy is kept separate from
//! allocation so it can be exhaustively tested without a Metal device.

use crate::RendererError;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTexture, MTLTextureDescriptor, MTLTextureType,
    MTLTextureUsage,
};

pub(crate) const GRADIENT_TEXTURE_WIDTH: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GradientResourceDescriptor {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) pixel_format: MTLPixelFormat,
    pub(crate) storage_mode: MTLStorageMode,
    pub(crate) texture_type: MTLTextureType,
    pub(crate) mipmap_level_count: usize,
    pub(crate) usage: MTLTextureUsage,
}

impl GradientResourceDescriptor {
    /// Build the exact descriptor requested by upstream.  A zero extent clears
    /// the resource and is represented by `None`; non-canonical widths are
    /// rejected because gradient addressing is defined for 512 columns.
    pub(crate) fn new(width: usize, height: usize) -> Option<Self> {
        if width == 0 || height == 0 || width != GRADIENT_TEXTURE_WIDTH {
            return None;
        }
        Some(Self {
            width,
            height,
            pixel_format: MTLPixelFormat::RGBA8Unorm,
            storage_mode: MTLStorageMode::Private,
            texture_type: MTLTextureType::Type2D,
            mipmap_level_count: 1,
            usage: MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead,
        })
    }
}

#[derive(Clone)]
pub(crate) struct GradientResource {
    descriptor: Option<GradientResourceDescriptor>,
    texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
}

impl GradientResource {
    pub(crate) fn new(
        device: &ProtocolObject<dyn MTLDevice>,
        width: usize,
        height: usize,
    ) -> Result<Option<Self>, RendererError> {
        let Some(descriptor) = GradientResourceDescriptor::new(width, height) else {
            return Ok(None);
        };
        let texture = make_texture(device, descriptor)?;
        Ok(Some(Self {
            descriptor: Some(descriptor),
            texture: Some(texture),
        }))
    }

    /// Resize with replacement-before-swap semantics.  Equal dimensions keep
    /// the existing Metal texture identity.  A zero extent clears the old
    /// owner and returns `None`.
    pub(crate) fn resize(
        &mut self,
        device: &ProtocolObject<dyn MTLDevice>,
        width: usize,
        height: usize,
    ) -> Result<Option<&ProtocolObject<dyn MTLTexture>>, RendererError> {
        let Some(descriptor) = GradientResourceDescriptor::new(width, height) else {
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
        descriptor: GradientResourceDescriptor,
        allocate: F,
    ) -> Result<Option<&ProtocolObject<dyn MTLTexture>>, RendererError>
    where
        F: FnOnce(
            GradientResourceDescriptor,
        ) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, RendererError>,
    {
        let replacement = allocate(descriptor)?;
        self.descriptor = Some(descriptor);
        self.texture = Some(replacement);
        Ok(self.texture.as_deref())
    }

    pub(crate) fn descriptor(&self) -> Option<GradientResourceDescriptor> {
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
    descriptor: GradientResourceDescriptor,
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
            RendererError::NativeMetal("failed to allocate gradient resource texture".to_owned())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_metal::MTLResource;

    #[test]
    fn descriptor_policy_matches_upstream_gradient_texture() {
        let descriptor = GradientResourceDescriptor::new(GRADIENT_TEXTURE_WIDTH, 17).unwrap();
        assert_eq!(descriptor.width, 512);
        assert_eq!(descriptor.height, 17);
        assert_eq!(descriptor.pixel_format, MTLPixelFormat::RGBA8Unorm);
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
        for extent in [(0, 1), (GRADIENT_TEXTURE_WIDTH, 0), (0, 0), (511, 3)] {
            assert_eq!(GradientResourceDescriptor::new(extent.0, extent.1), None);
        }
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_gradient_descriptor_and_resize_preserve_identity() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let mut resource = GradientResource::new(&device, GRADIENT_TEXTURE_WIDTH, 3)
            .unwrap()
            .unwrap();
        let first = resource.texture().unwrap() as *const ProtocolObject<dyn MTLTexture>;
        let same = resource
            .resize(&device, GRADIENT_TEXTURE_WIDTH, 3)
            .unwrap()
            .unwrap() as *const ProtocolObject<dyn MTLTexture>;
        assert!(std::ptr::eq(first, same));

        let replacement = resource
            .resize(&device, GRADIENT_TEXTURE_WIDTH, 4)
            .unwrap()
            .unwrap() as *const ProtocolObject<dyn MTLTexture>;
        assert!(!std::ptr::eq(first, replacement));
        assert_eq!(resource.descriptor().unwrap().height, 4);
        let texture = resource.texture().unwrap();
        assert_eq!(texture.width(), 512);
        assert_eq!(texture.height(), 4);
        assert_eq!(texture.pixelFormat(), MTLPixelFormat::RGBA8Unorm);
        assert_eq!(texture.storageMode(), MTLStorageMode::Private);
        assert_eq!(texture.textureType(), MTLTextureType::Type2D);
        assert_eq!(texture.mipmapLevelCount(), 1);
        assert_eq!(
            texture.usage(),
            MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead
        );

        assert!(resource
            .resize(&device, GRADIENT_TEXTURE_WIDTH, 0)
            .unwrap()
            .is_none());
        assert!(resource.texture().is_none());
        assert!(resource.descriptor().is_none());
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn failed_gradient_resize_preserves_identity_and_real_retry_succeeds() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let mut resource = GradientResource::new(&device, GRADIENT_TEXTURE_WIDTH, 1)
            .unwrap()
            .unwrap();
        let clone = resource.clone();
        let before = resource.texture().unwrap() as *const ProtocolObject<dyn MTLTexture>;
        assert!(std::ptr::eq(
            before,
            clone.texture().unwrap() as *const ProtocolObject<dyn MTLTexture>
        ));
        let before_descriptor = resource.descriptor();
        let replacement = GradientResourceDescriptor::new(GRADIENT_TEXTURE_WIDTH, 2).unwrap();

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
            .resize(&device, GRADIENT_TEXTURE_WIDTH, 2)
            .unwrap()
            .unwrap() as *const ProtocolObject<dyn MTLTexture>;
        assert!(!std::ptr::eq(before, after));
        assert_eq!(retained_before_retry.width(), 512);
        assert_eq!(retained_before_retry.height(), 1);
        assert_eq!(resource.descriptor().unwrap().height, 2);
        let same_shape = resource
            .resize(&device, GRADIENT_TEXTURE_WIDTH, 2)
            .unwrap()
            .unwrap() as *const ProtocolObject<dyn MTLTexture>;
        assert!(std::ptr::eq(after, same_shape));
    }
}
