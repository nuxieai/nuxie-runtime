//! Native Metal feather-atlas texture leaf translated from
//! `renderer/include/rive/renderer/metal/render_context_metal_impl.h:206,262`
//! and
//! `renderer/src/metal/render_context_metal_impl.mm:1079-1096`.
//!
//! Pinned upstream source: `rive-runtime` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
//!
//! The upstream owner stores a nullable `id<MTLTexture>` and replaces it on
//! every nonzero resize. The retained Rust owner keeps that lifecycle explicit
//! while making a zero extent the same absent-resource state.

use crate::RendererError;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTexture, MTLTextureDescriptor, MTLTextureType,
    MTLTextureUsage,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FeatherAtlasResourceDescriptor {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) pixel_format: MTLPixelFormat,
    pub(crate) storage_mode: MTLStorageMode,
    pub(crate) texture_type: MTLTextureType,
    pub(crate) mipmap_level_count: usize,
    pub(crate) usage: MTLTextureUsage,
}

impl FeatherAtlasResourceDescriptor {
    /// Build the exact descriptor requested by the pinned upstream leaf.
    /// Zero extents are the clear-resource case and are represented by
    /// `None`; all nonzero dimensions are valid policy inputs and Metal may
    /// still reject an allocation that exceeds the device limit.
    pub(crate) fn new(width: usize, height: usize) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        Some(Self {
            width,
            height,
            pixel_format: MTLPixelFormat::R16Float,
            storage_mode: MTLStorageMode::Private,
            texture_type: MTLTextureType::Type2D,
            mipmap_level_count: 1,
            usage: MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead,
        })
    }
}

/// Retained owner of the optional feather-atlas texture.
#[derive(Clone)]
pub(crate) struct FeatherAtlasResource {
    descriptor: Option<FeatherAtlasResourceDescriptor>,
    texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
}

impl FeatherAtlasResource {
    pub(crate) fn new(
        device: &ProtocolObject<dyn MTLDevice>,
        width: usize,
        height: usize,
    ) -> Result<Option<Self>, RendererError> {
        let Some(descriptor) = FeatherAtlasResourceDescriptor::new(width, height) else {
            return Ok(None);
        };
        let texture = make_texture(device, descriptor)?;
        Ok(Some(Self {
            descriptor: Some(descriptor),
            texture: Some(texture),
        }))
    }

    /// Force the pinned upstream resize operation with
    /// replacement-before-swap semantics. Every nonzero call creates a new
    /// texture, including equal dimensions; a zero extent clears the owner.
    pub(crate) fn replace(
        &mut self,
        device: &ProtocolObject<dyn MTLDevice>,
        width: usize,
        height: usize,
    ) -> Result<Option<&ProtocolObject<dyn MTLTexture>>, RendererError> {
        let Some(descriptor) = FeatherAtlasResourceDescriptor::new(width, height) else {
            self.descriptor = None;
            self.texture = None;
            return Ok(None);
        };
        self.replace_with(descriptor, |descriptor| make_texture(device, descriptor))
    }

    /// Replace a texture transactionally using the supplied allocator. The
    /// allocator runs before either owner is changed, so an allocation error
    /// leaves the current descriptor and texture identity untouched.
    fn replace_with<F>(
        &mut self,
        descriptor: FeatherAtlasResourceDescriptor,
        allocate: F,
    ) -> Result<Option<&ProtocolObject<dyn MTLTexture>>, RendererError>
    where
        F: FnOnce(
            FeatherAtlasResourceDescriptor,
        ) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, RendererError>,
    {
        let replacement = allocate(descriptor)?;
        self.descriptor = Some(descriptor);
        self.texture = Some(replacement);
        Ok(self.texture.as_deref())
    }

    /// Retain a fitting allocation across flushes, growing transactionally
    /// when the requested content no longer fits. Upstream makes this decision
    /// in its allocation policy before calling the forced resize leaf.
    pub(crate) fn ensure_capacity(
        &mut self,
        device: &ProtocolObject<dyn MTLDevice>,
        width: usize,
        height: usize,
    ) -> Result<Option<&ProtocolObject<dyn MTLTexture>>, RendererError> {
        let Some(requested) = FeatherAtlasResourceDescriptor::new(width, height) else {
            return Ok(self.texture.as_deref());
        };
        if self.descriptor.is_some_and(|current| {
            current.width >= requested.width && current.height >= requested.height
        }) && self.texture.is_some()
        {
            return Ok(self.texture.as_deref());
        }
        let [replacement_width, replacement_height] =
            self.descriptor
                .map_or([requested.width, requested.height], |current| {
                    [
                        current.width.max(requested.width),
                        current.height.max(requested.height),
                    ]
                });
        self.replace(device, replacement_width, replacement_height)
    }

    #[cfg(test)]
    pub(crate) fn descriptor(&self) -> Option<FeatherAtlasResourceDescriptor> {
        self.descriptor
    }

    #[cfg(test)]
    pub(crate) fn texture(&self) -> Option<&ProtocolObject<dyn MTLTexture>> {
        self.texture.as_deref()
    }

    pub(crate) fn retained_texture(&self) -> Option<Retained<ProtocolObject<dyn MTLTexture>>> {
        self.texture.clone()
    }
}

fn make_texture(
    device: &ProtocolObject<dyn MTLDevice>,
    descriptor: FeatherAtlasResourceDescriptor,
) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, RendererError> {
    let metal_descriptor = MTLTextureDescriptor::new();
    metal_descriptor.setPixelFormat(descriptor.pixel_format);
    // SAFETY: descriptor policy excludes zero dimensions, and these usize
    // values are passed directly to objc2's typed setters. The setters retain
    // no Rust pointers; Metal validates the dimensions and returns `None`
    // when the device cannot realize the requested texture.
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
                "failed to allocate feather atlas resource texture".to_owned(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_metal::MTLResource;

    #[test]
    fn descriptor_policy_matches_upstream_feather_atlas_texture() {
        let descriptor = FeatherAtlasResourceDescriptor::new(37, 19).unwrap();
        assert_eq!(descriptor.width, 37);
        assert_eq!(descriptor.height, 19);
        assert_eq!(descriptor.pixel_format, MTLPixelFormat::R16Float);
        assert_eq!(descriptor.storage_mode, MTLStorageMode::Private);
        assert_eq!(descriptor.texture_type, MTLTextureType::Type2D);
        assert_eq!(descriptor.mipmap_level_count, 1);
        assert_eq!(
            descriptor.usage,
            MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead
        );
    }

    #[test]
    fn zero_extent_has_no_descriptor() {
        for extent in [(0, 1), (1, 0), (0, 0)] {
            assert_eq!(
                FeatherAtlasResourceDescriptor::new(extent.0, extent.1),
                None
            );
        }
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_feather_atlas_capacity_reuses_and_forced_resize_replaces() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let mut resource = FeatherAtlasResource::new(&device, 37, 19).unwrap().unwrap();
        let first = resource.texture().unwrap() as *const ProtocolObject<dyn MTLTexture>;
        let same = resource.ensure_capacity(&device, 37, 19).unwrap().unwrap()
            as *const ProtocolObject<dyn MTLTexture>;
        assert!(std::ptr::eq(first, same));

        let forced = resource.replace(&device, 37, 19).unwrap().unwrap()
            as *const ProtocolObject<dyn MTLTexture>;
        assert!(!std::ptr::eq(first, forced));

        let replacement = resource.ensure_capacity(&device, 41, 23).unwrap().unwrap()
            as *const ProtocolObject<dyn MTLTexture>;
        assert!(!std::ptr::eq(forced, replacement));
        assert_eq!(resource.descriptor().unwrap().width, 41);
        assert_eq!(resource.descriptor().unwrap().height, 23);

        resource.ensure_capacity(&device, 47, 11).unwrap();
        assert_eq!(resource.descriptor().unwrap().width, 47);
        assert_eq!(resource.descriptor().unwrap().height, 23);
        let texture = resource.texture().unwrap();
        assert_eq!(texture.width(), 47);
        assert_eq!(texture.height(), 23);
        assert_eq!(texture.pixelFormat(), MTLPixelFormat::R16Float);
        assert_eq!(texture.storageMode(), MTLStorageMode::Private);
        assert_eq!(texture.textureType(), MTLTextureType::Type2D);
        assert_eq!(texture.mipmapLevelCount(), 1);
        assert_eq!(
            texture.usage(),
            MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead
        );

        assert!(resource.replace(&device, 0, 23).unwrap().is_none());
        assert!(resource.texture().is_none());
        assert!(resource.descriptor().is_none());
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_feather_atlas_drop_releases_owner_without_invalidating_retained_texture() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let resource = FeatherAtlasResource::new(&device, 5, 7).unwrap().unwrap();
        let retained = resource.retained_texture().unwrap();
        drop(resource);
        assert_eq!(retained.width(), 5);
        assert_eq!(retained.height(), 7);
        assert_eq!(retained.pixelFormat(), MTLPixelFormat::R16Float);
        drop(retained);
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn failed_feather_atlas_replace_preserves_identity_and_real_retry_succeeds() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let mut resource = FeatherAtlasResource::new(&device, 37, 19).unwrap().unwrap();
        let clone = resource.clone();
        let before = resource.texture().unwrap() as *const ProtocolObject<dyn MTLTexture>;
        assert!(std::ptr::eq(
            before,
            clone.texture().unwrap() as *const ProtocolObject<dyn MTLTexture>
        ));
        let before_descriptor = resource.descriptor();
        let replacement = FeatherAtlasResourceDescriptor::new(41, 23).unwrap();

        let failed = resource.replace_with(replacement, |_| {
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
        let after = resource.replace(&device, 41, 23).unwrap().unwrap()
            as *const ProtocolObject<dyn MTLTexture>;
        assert!(!std::ptr::eq(before, after));
        assert_eq!(retained_before_retry.width(), 37);
        assert_eq!(resource.descriptor().unwrap().width, 41);
        assert_eq!(resource.descriptor().unwrap().height, 23);
    }
}
