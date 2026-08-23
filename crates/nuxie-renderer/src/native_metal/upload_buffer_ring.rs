//! Native Metal upload-buffer ring.
//!
//! Mechanical translation of the pinned upstream implementation in
//! `renderer/include/rive/renderer/buffer_ring.hpp:22-80` and
//! `renderer/src/metal/render_context_metal_impl.mm:414-452` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
//!
//! The upstream ring starts with submitted-buffer index zero and advances the
//! index before each map.  Consequently, the first three maps select physical
//! slots one, two, and zero.  Metal's shared storage is directly CPU-visible;
//! an upload is submitted by ending the map and recording that slot as the
//! most recently submitted one.

use std::fmt;
use std::slice;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLBuffer, MTLDevice, MTLResourceOptions};

use crate::RendererError;

/// The ring size used by the pinned upstream `gpu::kBufferRingSize`.
pub(crate) const UPLOAD_BUFFER_RING_SIZE: usize = 3;

/// Errors from the stateful upload-buffer-ring protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UploadBufferRingError {
    /// A second map was attempted before the current map was submitted.
    AlreadyMapped,
    /// An unmap-and-submit was attempted without an active map.
    NotMapped,
    /// Metal map requests must contain at least one byte.
    ZeroRequiredBytes,
    /// The requested map length exceeds the ring's verbatim capacity.
    RequiredBytesExceedCapacity {
        required_bytes: usize,
        capacity: usize,
    },
}

impl fmt::Display for UploadBufferRingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyMapped => f.write_str("native Metal upload ring is already mapped"),
            Self::NotMapped => f.write_str("native Metal upload ring is not mapped"),
            Self::ZeroRequiredBytes => {
                f.write_str("native Metal upload map requires nonzero bytes")
            }
            Self::RequiredBytesExceedCapacity {
                required_bytes,
                capacity,
            } => write!(
                f,
                "native Metal upload map requires {required_bytes} bytes, capacity is {capacity}"
            ),
        }
    }
}

impl std::error::Error for UploadBufferRingError {}

/// Three shared Metal buffers used for CPU uploads while a prior frame may be
/// consumed by the GPU.
pub(crate) struct UploadBufferRing {
    // Rust lifetime adaptation: upstream's `BufferRingMetalImpl` relies on its
    // containing `RenderContextMetalImpl::m_gpu`; the standalone Rust owner
    // explicitly retains that same device so it cannot outlive the constructor
    // borrow in focused tests or a future extracted owner.
    _device: Retained<ProtocolObject<dyn MTLDevice>>,
    buffers: [Retained<ProtocolObject<dyn MTLBuffer>>; UPLOAD_BUFFER_RING_SIZE],
    capacity: usize,
    cursor: usize,
    mapped_slot: Option<usize>,
    submitted_slot: Option<usize>,
}

impl UploadBufferRing {
    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }

    #[cfg(test)]
    pub(crate) fn submitted_slot(&self) -> Option<usize> {
        self.submitted_slot
    }

    #[cfg(test)]
    pub(crate) fn buffer_identity(&self, slot: usize) -> *const () {
        Retained::as_ptr(&self.buffers[slot]) as *const ()
    }

    /// Allocate an upstream-compatible ring.
    ///
    /// A zero capacity follows `BufferRingMetalImpl::Make` and returns
    /// `Ok(None)`.  Nonzero capacities are passed verbatim to Metal; no
    /// alignment or rounding is introduced at this layer.
    pub(crate) fn new(
        device: &ProtocolObject<dyn MTLDevice>,
        capacity: usize,
    ) -> Result<Option<Self>, RendererError> {
        if capacity == 0 {
            return Ok(None);
        }

        let device_pointer = std::ptr::from_ref(device).cast_mut();
        // SAFETY: `device_pointer` is derived from a live borrowed Objective-C
        // protocol object. Retaining it extends the device lifetime to cover
        // the three buffers owned by this standalone Rust ring; production's
        // containing `NativeMetalContext` retains the same device separately.
        let retained_device = unsafe { Retained::retain(device_pointer) }
            .expect("a pointer derived from a reference is non-null");

        let mut buffers = Vec::with_capacity(UPLOAD_BUFFER_RING_SIZE);
        for _ in 0..UPLOAD_BUFFER_RING_SIZE {
            let buffer = device
                .newBufferWithLength_options(capacity, MTLResourceOptions::StorageModeShared)
                .ok_or_else(|| {
                    RendererError::NativeMetal(format!(
                        "failed to allocate shared upload buffer ({capacity} bytes)"
                    ))
                })?;
            buffers.push(buffer);
        }
        let buffers: [Retained<ProtocolObject<dyn MTLBuffer>>; UPLOAD_BUFFER_RING_SIZE] = buffers
            .try_into()
            .expect("the upload ring allocates exactly three buffers");

        Ok(Some(Self {
            _device: retained_device,
            buffers,
            capacity,
            // Upstream `m_submittedBufferIdx` starts at zero and is advanced
            // before `onMapBuffer`, so slot one is the first mapped slot.
            cursor: 0,
            mapped_slot: None,
            submitted_slot: None,
        }))
    }

    /// Map the next physical slot for `required_bytes` CPU-visible bytes.
    pub(crate) fn map(
        &mut self,
        required_bytes: usize,
    ) -> Result<&mut [u8], UploadBufferRingError> {
        if self.mapped_slot.is_some() {
            return Err(UploadBufferRingError::AlreadyMapped);
        }
        if required_bytes == 0 {
            return Err(UploadBufferRingError::ZeroRequiredBytes);
        }
        if required_bytes > self.capacity {
            return Err(UploadBufferRingError::RequiredBytesExceedCapacity {
                required_bytes,
                capacity: self.capacity,
            });
        }

        let slot = (self.cursor + 1) % UPLOAD_BUFFER_RING_SIZE;
        self.cursor = slot;
        self.mapped_slot = Some(slot);
        let buffer = &self.buffers[slot];
        let pointer = buffer.contents();
        // SAFETY: every buffer is created with `StorageModeShared` and the
        // exact `capacity` used here. The returned slice is limited to the
        // validated request and cannot outlive the mutable borrow of `self`.
        Ok(unsafe { slice::from_raw_parts_mut(pointer.as_ptr().cast(), required_bytes) })
    }

    /// End the current map and make its physical slot the submitted buffer.
    pub(crate) fn unmap_submit(&mut self) -> Result<(), UploadBufferRingError> {
        let slot = self
            .mapped_slot
            .take()
            .ok_or(UploadBufferRingError::NotMapped)?;
        self.submitted_slot = Some(slot);
        Ok(())
    }

    /// Retain the most recently submitted physical buffer.
    ///
    /// The result is deliberately an error before the first submission: a
    /// caller must not bind an arbitrary ring slot as if it contained valid
    /// upload data.
    pub(crate) fn retained_submitted_buffer(
        &self,
    ) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, RendererError> {
        if self.mapped_slot.is_some() {
            return Err(RendererError::NativeMetal(
                "native Metal upload ring cannot expose a submitted buffer while mapped".to_owned(),
            ));
        }
        let slot = self.submitted_slot.ok_or_else(|| {
            RendererError::NativeMetal(
                "native Metal upload ring has no submitted buffer".to_owned(),
            )
        })?;
        Ok(self.buffers[slot].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn make_ring() -> Option<UploadBufferRing> {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return None;
        };
        UploadBufferRing::new(&device, 16).expect("shared upload ring allocation")
    }

    fn bytes(buffer: &ProtocolObject<dyn MTLBuffer>, length: usize) -> Vec<u8> {
        // SAFETY: this helper is used only with shared buffers allocated by
        // `UploadBufferRing`; the caller supplies a length within capacity.
        unsafe { slice::from_raw_parts(buffer.contents().as_ptr().cast(), length).to_vec() }
    }

    #[test]
    fn zero_capacity_mirrors_upstream_null_ring() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        assert!(UploadBufferRing::new(&device, 0).unwrap().is_none());
    }

    #[test]
    fn map_rolls_over_one_two_zero() {
        let Some(mut ring) = make_ring() else {
            return;
        };
        let mut selected = Vec::new();
        for value in [1_u8, 2, 3] {
            let mapped = ring.map(1).unwrap();
            mapped[0] = value;
            selected.push(ring.cursor);
            ring.unmap_submit().unwrap();
        }
        assert_eq!(selected, [1, 2, 0]);
    }

    #[test]
    fn double_map_is_rejected_without_advancing_the_ring() {
        let Some(mut ring) = make_ring() else {
            return;
        };
        ring.map(1).unwrap()[0] = 7;
        assert_eq!(ring.map(1), Err(UploadBufferRingError::AlreadyMapped));
        assert_eq!(ring.cursor, 1);
        ring.unmap_submit().unwrap();
    }

    #[test]
    fn zero_and_over_capacity_maps_are_rejected() {
        let Some(mut ring) = make_ring() else {
            return;
        };
        assert_eq!(ring.map(0), Err(UploadBufferRingError::ZeroRequiredBytes));
        assert_eq!(
            ring.map(17),
            Err(UploadBufferRingError::RequiredBytesExceedCapacity {
                required_bytes: 17,
                capacity: 16,
            })
        );
        assert_eq!(ring.cursor, 0);
    }

    #[test]
    fn submit_unmapped_and_read_before_first_submit_are_rejected() {
        let Some(mut ring) = make_ring() else {
            return;
        };
        assert_eq!(ring.unmap_submit(), Err(UploadBufferRingError::NotMapped));
        assert!(ring.retained_submitted_buffer().is_err());
        ring.map(1).unwrap()[0] = 1;
        assert!(ring.retained_submitted_buffer().is_err());
        ring.unmap_submit().unwrap();
    }

    #[test]
    fn rings_rotate_independently() {
        let Some(mut first) = make_ring() else {
            return;
        };
        let Some(mut second) = make_ring() else {
            return;
        };
        first.map(1).unwrap()[0] = 1;
        first.unmap_submit().unwrap();
        second.map(1).unwrap()[0] = 2;
        second.unmap_submit().unwrap();
        assert_eq!(first.cursor, 1);
        assert_eq!(second.cursor, 1);
        first.map(1).unwrap()[0] = 3;
        first.unmap_submit().unwrap();
        assert_eq!(first.cursor, 2);
        assert_eq!(second.cursor, 1);
    }

    #[test]
    fn reports_verbatim_capacity_and_submitted_slot() {
        let Some(mut ring) = make_ring() else {
            return;
        };
        assert_eq!(ring.capacity(), 16);
        assert_eq!(ring.submitted_slot(), None);
        ring.map(1).unwrap()[0] = 9;
        ring.unmap_submit().unwrap();
        assert_eq!(ring.submitted_slot(), Some(1));
    }

    #[test]
    fn live_shared_slots_retain_distinct_uploaded_bytes() {
        let Some(mut ring) = make_ring() else {
            return;
        };
        let mut submitted = Vec::new();
        let mut pointers = Vec::new();
        for value in [0x11_u8, 0x22, 0x33] {
            ring.map(4).unwrap().fill(value);
            ring.unmap_submit().unwrap();
            let buffer = ring.retained_submitted_buffer().unwrap();
            pointers.push(buffer.as_ref() as *const ProtocolObject<dyn MTLBuffer>);
            submitted.push(buffer);
        }
        assert_eq!(pointers.len(), UPLOAD_BUFFER_RING_SIZE);
        assert_ne!(pointers[0], pointers[1]);
        assert_ne!(pointers[1], pointers[2]);
        assert_ne!(pointers[0], pointers[2]);
        assert_eq!(bytes(&submitted[0], 4), [0x11; 4]);
        assert_eq!(bytes(&submitted[1], 4), [0x22; 4]);
        assert_eq!(bytes(&submitted[2], 4), [0x33; 4]);
    }

    #[test]
    fn invalid_sequences_do_not_panic() {
        let Some(mut ring) = make_ring() else {
            return;
        };
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _ = ring.unmap_submit();
        }))
        .is_ok());
    }
}
