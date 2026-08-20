//! Mechanical Rust translation of the pinned upstream Metal render target.
//!
//! The ownership and attachment rules are translated from:
//! - `renderer/include/rive/renderer/metal/render_context_metal_impl.h:21-87`
//! - `renderer/src/metal/render_context_metal_impl.mm:735-781`
//!
//! Pinned upstream source: `rive-runtime` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

use super::capabilities::MetalCapabilitySelection;
use crate::RendererError;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLBuffer, MTLDevice, MTLPixelFormat, MTLResourceOptions, MTLStorageMode, MTLTexture,
    MTLTextureDescriptor, MTLTextureType, MTLTextureUsage,
};

/// The PLS attachment formats selected by the upstream raster-order path.
///
/// Coverage and clip are integer planes. The scratch color plane has the
/// render target's color format. Atomic mode does not create these textures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RasterOrderAttachmentFormats {
    coverage: MTLPixelFormat,
    clip: MTLPixelFormat,
    scratch_color: MTLPixelFormat,
}

fn raster_order_attachment_formats(
    supports_raster_ordering: bool,
    pixel_format: MTLPixelFormat,
) -> Option<RasterOrderAttachmentFormats> {
    supports_raster_ordering.then_some(RasterOrderAttachmentFormats {
        coverage: MTLPixelFormat::R32Uint,
        clip: MTLPixelFormat::R32Uint,
        scratch_color: pixel_format,
    })
}

fn compatible_texture_properties(
    expected_width: u32,
    expected_height: u32,
    expected_pixel_format: MTLPixelFormat,
    texture_width: usize,
    texture_height: usize,
    texture_pixel_format: MTLPixelFormat,
    texture_usage: MTLTextureUsage,
) -> bool {
    texture_usage.contains(MTLTextureUsage::RenderTarget)
        && texture_width == expected_width as usize
        && texture_height == expected_height as usize
        && texture_pixel_format == expected_pixel_format
}

fn atomic_buffer_size(width: u32, height: u32) -> Result<usize, RendererError> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(std::mem::size_of::<u32>()))
        .ok_or_else(|| RendererError::NativeMetal("atomic buffer size overflow".to_owned()))
}

fn validate_render_target_extent(width: u32, height: u32) -> Result<(), RendererError> {
    if width == 0 || height == 0 {
        return Err(RendererError::NativeMetal(format!(
            "render target dimensions must be at least 1x1 (got {width}x{height})"
        )));
    }
    Ok(())
}

fn make_pls_memoryless_texture(
    device: &ProtocolObject<dyn MTLDevice>,
    pixel_format: MTLPixelFormat,
    width: u32,
    height: u32,
) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, RendererError> {
    // This is the Rust equivalent of the Objective-C descriptor allocated by
    // make_pls_memoryless_texture() in the pinned upstream implementation.
    let descriptor = MTLTextureDescriptor::new();
    descriptor.setPixelFormat(pixel_format);
    // SAFETY: `RenderTargetMetal::new` validates width and height are >= 1
    // before calling this helper, satisfying objc2's size precondition. Both
    // values widen losslessly from `u32`, the single mip level is valid for a
    // 2D texture, and these setters retain no Rust pointers. Metal validates
    // the typed pixel-format/dimension combination when
    // `newTextureWithDescriptor` is called and reports failure as `nil`.
    unsafe {
        descriptor.setWidth(width as usize);
        descriptor.setHeight(height as usize);
        descriptor.setMipmapLevelCount(1);
    }
    descriptor.setUsage(MTLTextureUsage::RenderTarget);
    descriptor.setTextureType(MTLTextureType::Type2D);
    descriptor.setStorageMode(MTLStorageMode::Memoryless);

    device.newTextureWithDescriptor(&descriptor).ok_or_else(|| {
        RendererError::NativeMetal("failed to allocate memoryless PLS texture".into())
    })
}

/// Metal backend implementation of the upstream `RenderTarget`.
///
/// The target owns the device, optional externally supplied target texture,
/// raster-order memoryless attachments, and lazy atomic buffers for its full
/// lifetime. Size-dependent resources are intentionally kept together so a
/// caller can replace the complete owner when resizing.
pub(crate) struct RenderTargetMetal {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    pixel_format: MTLPixelFormat,
    width: u32,
    height: u32,

    target_texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,

    coverage_memoryless_texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
    clip_memoryless_texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
    scratch_color_memoryless_texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,

    // Unlike memoryless textures, these buffers have physical storage. Keep
    // them nil until the atomic path first asks for each plane, as upstream
    // does with its Objective-C id fields.
    color_atomic_buffer: Option<Retained<ProtocolObject<dyn MTLBuffer>>>,
    coverage_atomic_buffer: Option<Retained<ProtocolObject<dyn MTLBuffer>>>,
    clip_atomic_buffer: Option<Retained<ProtocolObject<dyn MTLBuffer>>>,
}

impl RenderTargetMetal {
    /// Creates a target and its raster-order memoryless attachments.
    ///
    /// The device is accepted by value to make the target's retained-device
    /// ownership explicit. Atomic buffers remain lazy and are not allocated by
    /// this constructor.
    pub(crate) fn new(
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        pixel_format: MTLPixelFormat,
        width: u32,
        height: u32,
        capabilities: MetalCapabilitySelection,
    ) -> Result<Self, RendererError> {
        validate_render_target_extent(width, height)?;

        let formats =
            raster_order_attachment_formats(capabilities.supports_raster_ordering, pixel_format);
        let (
            coverage_memoryless_texture,
            clip_memoryless_texture,
            scratch_color_memoryless_texture,
        ) = match formats {
            Some(formats) => (
                Some(make_pls_memoryless_texture(
                    &device,
                    formats.coverage,
                    width,
                    height,
                )?),
                Some(make_pls_memoryless_texture(
                    &device,
                    formats.clip,
                    width,
                    height,
                )?),
                Some(make_pls_memoryless_texture(
                    &device,
                    formats.scratch_color,
                    width,
                    height,
                )?),
            ),
            None => (None, None, None),
        };

        Ok(Self {
            device,
            pixel_format,
            width,
            height,
            target_texture: None,
            coverage_memoryless_texture,
            clip_memoryless_texture,
            scratch_color_memoryless_texture,
            color_atomic_buffer: None,
            coverage_atomic_buffer: None,
            clip_atomic_buffer: None,
        })
    }

    pub(crate) fn device(&self) -> &ProtocolObject<dyn MTLDevice> {
        &self.device
    }

    pub(crate) fn pixel_format(&self) -> MTLPixelFormat {
        self.pixel_format
    }

    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    pub(crate) fn height(&self) -> u32 {
        self.height
    }

    /// Returns whether a texture can be used as this target's render target.
    ///
    /// Upstream asserts render-target usage before comparing dimensions and
    /// format. The Rust port treats an absent usage bit as incompatible so a
    /// malformed external texture fails closed without panicking.
    pub(crate) fn compatible_with(&self, texture: &ProtocolObject<dyn MTLTexture>) -> bool {
        compatible_texture_properties(
            self.width,
            self.height,
            self.pixel_format,
            texture.width(),
            texture.height(),
            texture.pixelFormat(),
            texture.usage(),
        )
    }

    /// Replaces the externally owned target texture after compatibility check.
    /// Passing `None` detaches the target, matching upstream's nullable setter.
    pub(crate) fn set_target_texture(
        &mut self,
        texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
    ) -> Result<(), RendererError> {
        if let Some(texture) = texture.as_deref() {
            if !self.compatible_with(texture) {
                return Err(RendererError::NativeMetal(
                    "target texture is incompatible with render target".to_owned(),
                ));
            }
        }
        self.target_texture = texture;
        Ok(())
    }

    pub(crate) fn target_texture(&self) -> Option<&ProtocolObject<dyn MTLTexture>> {
        self.target_texture.as_deref()
    }

    pub(crate) fn retained_target_texture(
        &self,
    ) -> Option<Retained<ProtocolObject<dyn MTLTexture>>> {
        self.target_texture.clone()
    }

    pub(crate) fn coverage_memoryless_texture(&self) -> Option<&ProtocolObject<dyn MTLTexture>> {
        self.coverage_memoryless_texture.as_deref()
    }

    pub(crate) fn clip_memoryless_texture(&self) -> Option<&ProtocolObject<dyn MTLTexture>> {
        self.clip_memoryless_texture.as_deref()
    }

    pub(crate) fn scratch_color_memoryless_texture(
        &self,
    ) -> Option<&ProtocolObject<dyn MTLTexture>> {
        self.scratch_color_memoryless_texture.as_deref()
    }

    fn make_atomic_buffer(&self) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, RendererError> {
        let length = atomic_buffer_size(self.width, self.height)?;
        self.device
            .newBufferWithLength_options(length, MTLResourceOptions::StorageModePrivate)
            .ok_or_else(|| {
                RendererError::NativeMetal("failed to allocate private atomic buffer".into())
            })
    }

    pub(crate) fn color_atomic_buffer(
        &mut self,
    ) -> Result<&ProtocolObject<dyn MTLBuffer>, RendererError> {
        if self.color_atomic_buffer.is_none() {
            self.color_atomic_buffer = Some(self.make_atomic_buffer()?);
        }
        Ok(self
            .color_atomic_buffer
            .as_deref()
            .expect("atomic buffer initialized above"))
    }

    pub(crate) fn coverage_atomic_buffer(
        &mut self,
    ) -> Result<&ProtocolObject<dyn MTLBuffer>, RendererError> {
        if self.coverage_atomic_buffer.is_none() {
            self.coverage_atomic_buffer = Some(self.make_atomic_buffer()?);
        }
        Ok(self
            .coverage_atomic_buffer
            .as_deref()
            .expect("atomic buffer initialized above"))
    }

    pub(crate) fn clip_atomic_buffer(
        &mut self,
    ) -> Result<&ProtocolObject<dyn MTLBuffer>, RendererError> {
        if self.clip_atomic_buffer.is_none() {
            self.clip_atomic_buffer = Some(self.make_atomic_buffer()?);
        }
        Ok(self
            .clip_atomic_buffer
            .as_deref()
            .expect("atomic buffer initialized above"))
    }

    pub(crate) fn atomic_plane_inventory(&self) -> [bool; 3] {
        [
            self.color_atomic_buffer.is_some(),
            self.clip_atomic_buffer.is_some(),
            self.coverage_atomic_buffer.is_some(),
        ]
    }

    #[cfg(test)]
    fn atomic_buffers_are_unallocated(&self) -> bool {
        self.color_atomic_buffer.is_none()
            && self.coverage_atomic_buffer.is_none()
            && self.clip_atomic_buffer.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_metal::MTLResource;

    #[test]
    fn compatibility_policy_requires_render_target_usage_dimensions_and_format() {
        let expected_format = MTLPixelFormat::BGRA8Unorm;
        let cases = [
            (
                "compatible",
                64,
                32,
                expected_format,
                MTLTextureUsage::RenderTarget,
                true,
            ),
            (
                "shader-only texture",
                64,
                32,
                expected_format,
                MTLTextureUsage::ShaderRead,
                false,
            ),
            (
                "wrong width",
                63,
                32,
                expected_format,
                MTLTextureUsage::RenderTarget,
                false,
            ),
            (
                "wrong height",
                64,
                31,
                expected_format,
                MTLTextureUsage::RenderTarget,
                false,
            ),
            (
                "wrong format",
                64,
                32,
                MTLPixelFormat::RGBA8Unorm,
                MTLTextureUsage::RenderTarget,
                false,
            ),
        ];

        for (name, width, height, format, usage, expected) in cases {
            assert_eq!(
                compatible_texture_properties(
                    64,
                    32,
                    expected_format,
                    width as usize,
                    height as usize,
                    format,
                    usage,
                ),
                expected,
                "compatibility policy mismatch for {name}",
            );
        }
    }

    #[test]
    fn atomic_buffer_size_is_checked_before_allocation() {
        assert_eq!(atomic_buffer_size(2, 3).unwrap(), 24);
        assert!(matches!(
            atomic_buffer_size(u32::MAX, u32::MAX),
            Err(RendererError::NativeMetal(message)) if message == "atomic buffer size overflow"
        ));
    }

    #[test]
    fn render_target_extent_requires_nonzero_dimensions() {
        for (width, height) in [(0, 1), (1, 0)] {
            assert!(matches!(
                validate_render_target_extent(width, height),
                Err(RendererError::NativeMetal(message))
                    if message
                        == format!(
                            "render target dimensions must be at least 1x1 (got {width}x{height})"
                        )
            ));
        }

        assert!(validate_render_target_extent(1, 1).is_ok());
        assert!(validate_render_target_extent(u32::MAX, u32::MAX).is_ok());
    }

    #[test]
    fn raster_order_attachment_selection_matches_upstream_table() {
        let color_format = MTLPixelFormat::BGRA8Unorm;
        assert_eq!(raster_order_attachment_formats(false, color_format), None);
        assert_eq!(
            raster_order_attachment_formats(true, color_format),
            Some(RasterOrderAttachmentFormats {
                coverage: MTLPixelFormat::R32Uint,
                clip: MTLPixelFormat::R32Uint,
                scratch_color: color_format,
            })
        );
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn atomic_buffers_are_lazy_and_each_plane_keeps_identity() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            return;
        };
        let capabilities = MetalCapabilitySelection {
            supports_raster_ordering: false,
            ..capabilities_for_test()
        };
        let make_target = || {
            RenderTargetMetal::new(
                device.clone(),
                MTLPixelFormat::BGRA8Unorm,
                2,
                3,
                capabilities,
            )
            .unwrap()
        };

        let mut color_target = make_target();
        assert!(color_target.atomic_buffers_are_unallocated());
        let first_color_pointer = {
            let color = color_target.color_atomic_buffer().unwrap();
            assert_eq!(color.length(), 24);
            color as *const ProtocolObject<dyn MTLBuffer>
        };
        assert!(color_target.color_atomic_buffer.is_some());
        assert!(color_target.coverage_atomic_buffer.is_none());
        assert!(color_target.clip_atomic_buffer.is_none());
        let second_color_pointer = {
            let color = color_target.color_atomic_buffer().unwrap();
            color as *const ProtocolObject<dyn MTLBuffer>
        };
        assert!(std::ptr::eq(first_color_pointer, second_color_pointer));

        let mut coverage_target = make_target();
        assert!(coverage_target.atomic_buffers_are_unallocated());
        let first_coverage_pointer = {
            let coverage = coverage_target.coverage_atomic_buffer().unwrap();
            assert_eq!(coverage.length(), 24);
            coverage as *const ProtocolObject<dyn MTLBuffer>
        };
        assert!(coverage_target.color_atomic_buffer.is_none());
        assert!(coverage_target.coverage_atomic_buffer.is_some());
        assert!(coverage_target.clip_atomic_buffer.is_none());
        let second_coverage_pointer = {
            let coverage = coverage_target.coverage_atomic_buffer().unwrap();
            coverage as *const ProtocolObject<dyn MTLBuffer>
        };
        assert!(std::ptr::eq(
            first_coverage_pointer,
            second_coverage_pointer
        ));

        let mut clip_target = make_target();
        assert!(clip_target.atomic_buffers_are_unallocated());
        let first_clip_pointer = {
            let clip = clip_target.clip_atomic_buffer().unwrap();
            assert_eq!(clip.length(), 24);
            clip as *const ProtocolObject<dyn MTLBuffer>
        };
        assert!(clip_target.color_atomic_buffer.is_none());
        assert!(clip_target.coverage_atomic_buffer.is_none());
        assert!(clip_target.clip_atomic_buffer.is_some());
        let second_clip_pointer = {
            let clip = clip_target.clip_atomic_buffer().unwrap();
            clip as *const ProtocolObject<dyn MTLBuffer>
        };
        assert!(std::ptr::eq(first_clip_pointer, second_clip_pointer));
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn raster_order_constructor_selects_memoryless_attachments() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            return;
        };
        let mut capabilities = capabilities_for_test();
        capabilities.supports_raster_ordering = true;
        let target =
            RenderTargetMetal::new(device, MTLPixelFormat::BGRA8Unorm, 2, 3, capabilities).unwrap();

        assert_memoryless_attachment(
            target.coverage_memoryless_texture().unwrap(),
            MTLPixelFormat::R32Uint,
        );
        assert_memoryless_attachment(
            target.clip_memoryless_texture().unwrap(),
            MTLPixelFormat::R32Uint,
        );
        assert_memoryless_attachment(
            target.scratch_color_memoryless_texture().unwrap(),
            MTLPixelFormat::BGRA8Unorm,
        );
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    fn assert_memoryless_attachment(
        texture: &ProtocolObject<dyn MTLTexture>,
        expected_pixel_format: MTLPixelFormat,
    ) {
        assert_eq!(texture.storageMode(), MTLStorageMode::Memoryless);
        assert_eq!(texture.pixelFormat(), expected_pixel_format);
        assert_eq!(texture.width(), 2);
        assert_eq!(texture.height(), 3);
        assert_eq!(texture.usage(), MTLTextureUsage::RenderTarget);
        assert_eq!(texture.textureType(), MTLTextureType::Type2D);
        assert_eq!(texture.mipmapLevelCount(), 1);
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    fn capabilities_for_test() -> MetalCapabilitySelection {
        MetalCapabilitySelection {
            max_texture_size: 16_384,
            supports_raster_ordering: false,
            supports_atomic_mode: true,
            path_id_granularity: 1,
            supports_texture_compression_etc2: false,
            supports_texture_compression_astc: false,
            supports_texture_compression_bc: false,
            atomic_barrier_type: super::super::capabilities::AtomicBarrierType::RenderPassBreak,
        }
    }
}
