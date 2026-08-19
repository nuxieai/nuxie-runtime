use std::any::Any;

use nuxie_render_api::{RenderBuffer, RenderBufferFlags, RenderBufferType};

/// CPU staging storage for a future native Metal render buffer.
///
/// This type intentionally owns no `MTLBuffer` yet. It preserves the public
/// byte-oriented mapping contract while buffer upload and mesh draws are still
/// fail-closed in the tracer. Mapping returns the complete allocation, and
/// unmapping does not invalidate or copy the bytes.
pub(crate) struct NativeMetalBuffer {
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    bytes: Vec<u8>,
}

impl NativeMetalBuffer {
    pub(crate) fn new(
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Self {
        Self {
            buffer_type,
            flags,
            bytes: vec![0; size_in_bytes],
        }
    }
}

impl RenderBuffer for NativeMetalBuffer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn buffer_type(&self) -> RenderBufferType {
        self.buffer_type
    }

    fn flags(&self) -> RenderBufferFlags {
        self.flags
    }

    fn size_in_bytes(&self) -> usize {
        self.bytes.len()
    }

    fn map_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    fn unmap(&mut self) {
        // Shared Metal buffers remain CPU-visible after unmapping. Keeping
        // this operation idempotent preserves data across repeated calls.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_exposes_exact_allocation_bounds() {
        let mut buffer =
            NativeMetalBuffer::new(RenderBufferType::Vertex, RenderBufferFlags::None, 7);

        assert_eq!(buffer.size_in_bytes(), 7);
        assert_eq!(buffer.map_mut().len(), 7);
        buffer.unmap();
        buffer.unmap();
    }

    #[test]
    fn mapped_bytes_survive_unmap_and_remap() {
        let mut buffer = NativeMetalBuffer::new(
            RenderBufferType::Index,
            RenderBufferFlags::MappedOnceAtInitialization,
            4,
        );

        buffer.map_mut().copy_from_slice(&[0x01, 0x02, 0x03, 0x04]);
        buffer.unmap();

        assert_eq!(buffer.map_mut(), &[0x01, 0x02, 0x03, 0x04]);
        buffer.unmap();
    }

    #[test]
    fn zero_sized_mapping_is_safe() {
        let mut buffer =
            NativeMetalBuffer::new(RenderBufferType::Vertex, RenderBufferFlags::None, 0);

        assert!(buffer.map_mut().is_empty());
        buffer.unmap();
        assert!(buffer.map_mut().is_empty());
    }
}
