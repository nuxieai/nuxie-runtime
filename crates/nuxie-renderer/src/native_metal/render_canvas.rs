//! Same-texture render-target and sampled-image ownership.

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTexture, MTLTextureDescriptor, MTLTextureType,
    MTLTextureUsage,
};

use super::capabilities::MetalCapabilitySelection;
use super::image_texture::NativeMetalImageTexture;
use super::render_target::RenderTargetMetal;
use crate::RendererError;

/// One private RGBA texture viewed both as a render target and a sampled image.
///
/// This is the native owner created by the pinned Metal `makeRenderCanvas`
/// factory. It intentionally does not broaden the current tracer into image
/// compositing; callers can retain the paired owner until that flush family is
/// ported.
pub struct NativeMetalRenderCanvas {
    image: NativeMetalImageTexture,
    target: RenderTargetMetal,
}

impl NativeMetalRenderCanvas {
    pub(crate) fn new(
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        capabilities: MetalCapabilitySelection,
        width: u32,
        height: u32,
    ) -> Result<Self, RendererError> {
        let descriptor = MTLTextureDescriptor::new();
        descriptor.setPixelFormat(MTLPixelFormat::RGBA8Unorm);
        descriptor.setTextureType(MTLTextureType::Type2D);
        descriptor.setUsage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
        descriptor.setStorageMode(MTLStorageMode::Private);
        // SAFETY: the public factory validates nonzero dimensions against the
        // selected device limit. Both values widen losslessly to NSUInteger,
        // and the single mip level is valid for every nonzero 2D texture.
        unsafe {
            descriptor.setWidth(width as usize);
            descriptor.setHeight(height as usize);
            descriptor.setMipmapLevelCount(1);
        }
        let texture = device
            .newTextureWithDescriptor(&descriptor)
            .ok_or_else(|| RendererError::NativeMetal("failed to allocate render canvas".into()))?;

        let mut target = RenderTargetMetal::new(
            device,
            MTLPixelFormat::RGBA8Unorm,
            width,
            height,
            capabilities,
        )?;
        target.set_target_texture(Some(texture.clone()))?;
        let image =
            NativeMetalImageTexture::adopt(Some(texture), width, height).ok_or_else(|| {
                RendererError::NativeMetal("failed to adopt render canvas image".into())
            })?;
        Ok(Self { image, target })
    }

    pub fn width(&self) -> u32 {
        self.target.width()
    }

    pub fn height(&self) -> u32 {
        self.target.height()
    }

    /// Returns whether both wrappers still reference the exact same MTLTexture.
    pub fn render_target_and_image_share_texture(&self) -> bool {
        let Some(target) = self.target.target_texture() else {
            return false;
        };
        std::ptr::eq(target, self.image.texture())
    }

    /// Retains the canvas texture for a same-device native consumer such as
    /// the opt-in ORE context.
    pub fn retained_metal_texture(&self) -> Retained<ProtocolObject<dyn MTLTexture>> {
        self.target
            .retained_target_texture()
            .expect("render canvas target texture is set at construction")
    }
}
