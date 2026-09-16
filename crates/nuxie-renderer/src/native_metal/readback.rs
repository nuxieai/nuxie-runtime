//! Validated caller-owned pixel storage for a presented Metal frame.

use crate::RendererError;
use objc2::Message;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLBlitCommandEncoder, MTLBuffer, MTLCommandBuffer, MTLCommandEncoder, MTLOrigin,
    MTLPixelFormat, MTLResource, MTLSize, MTLStorageMode, MTLTexture, MTLTextureType,
};

/// Retains a validated source and destination until the submission completes.
/// The destination remains caller-owned and can outlive drawable recycling.
pub struct NativeMetalReadback {
    source: Retained<ProtocolObject<dyn MTLTexture>>,
    buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    bytes_per_row: usize,
    bytes_per_image: usize,
}

impl NativeMetalReadback {
    pub fn new(
        source: &ProtocolObject<dyn MTLTexture>,
        buffer: &ProtocolObject<dyn MTLBuffer>,
        bytes_per_row: usize,
    ) -> Result<Self, RendererError> {
        let invalid = |message: &str| RendererError::InvalidDrawable(message.to_owned());
        if source.isFramebufferOnly()
            || source.pixelFormat() != MTLPixelFormat::BGRA8Unorm
            || source.textureType() != MTLTextureType::Type2D
            || source.sampleCount() != 1
            || source.width() == 0
            || source.height() == 0
        {
            return Err(invalid(
                "readback requires a readable, nonempty BGRA8 2D drawable",
            ));
        }
        if buffer.storageMode() != MTLStorageMode::Shared {
            return Err(invalid("readback buffer must use shared storage"));
        }
        if Retained::as_ptr(&source.device()) != Retained::as_ptr(&buffer.device()) {
            return Err(invalid(
                "readback buffer and drawable must use the same MTLDevice",
            ));
        }
        let minimum_row = source
            .width()
            .checked_mul(4)
            .ok_or_else(|| invalid("readback row size overflow"))?;
        if bytes_per_row < minimum_row || !bytes_per_row.is_multiple_of(256) {
            return Err(invalid(
                "readback row stride must cover the pixels and be a multiple of 256",
            ));
        }
        let bytes_per_image = bytes_per_row
            .checked_mul(source.height())
            .ok_or_else(|| invalid("readback image size overflow"))?;
        if buffer.length() < bytes_per_image {
            return Err(invalid("readback buffer is too small"));
        }
        Ok(Self {
            source: source.retain(),
            buffer: buffer.retain(),
            bytes_per_row,
            bytes_per_image,
        })
    }

    pub(super) fn encode(
        &self,
        command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    ) -> Result<(), RendererError> {
        let encoder = command_buffer.blitCommandEncoder().ok_or_else(|| {
            RendererError::NativeMetal("readback blit encoder is unavailable".into())
        })?;
        // Construction validates format, device, storage, row stride and capacity.
        // The owning submission orders this copy after rendering and before present.
        unsafe {
            encoder.copyFromTexture_sourceSlice_sourceLevel_sourceOrigin_sourceSize_toBuffer_destinationOffset_destinationBytesPerRow_destinationBytesPerImage(
                &self.source, 0, 0, MTLOrigin { x: 0, y: 0, z: 0 },
                MTLSize { width: self.source.width(), height: self.source.height(), depth: 1 },
                &self.buffer, 0, self.bytes_per_row, self.bytes_per_image,
            );
        }
        encoder.endEncoding();
        Ok(())
    }
}
