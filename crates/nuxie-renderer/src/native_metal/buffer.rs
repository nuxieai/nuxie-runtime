//! Native Metal render-buffer leaf.
//!
//! Mechanical translation of the pinned upstream declaration and
//! implementation in
//! `renderer/include/rive/renderer/metal/render_context_metal_impl.h:135-137`
//! and
//! `renderer/src/metal/render_context_metal_impl.mm:783-828` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

use std::any::Any;
use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::NonNull;
use std::slice;

use nuxie_render_api::{RenderBuffer, RenderBufferFlags, RenderBufferType};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLBuffer, MTLDevice, MTLResourceOptions};

use crate::RendererError;

/// The ring size used by the pinned upstream `gpu::kBufferRingSize`.
pub(crate) const BUFFER_RING_SIZE: usize = 3;

fn buffer_count_for_flags(flags: RenderBufferFlags) -> usize {
    if flags == RenderBufferFlags::MappedOnceAtInitialization {
        1
    } else {
        BUFFER_RING_SIZE
    }
}

/// CPU-visible Metal storage for a Rive render buffer.
///
/// `mappedOnceAtInitialization` uses one shared `MTLBuffer`; mutable buffers
/// use the upstream three-buffer ring. A map addresses the current back
/// buffer. The next submission makes a dirty back buffer the front identity
/// and advances the back index, matching `RiveRenderBuffer::frontBufferIdx()`.
pub(crate) struct NativeMetalBuffer {
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    size_in_bytes: usize,
    // Upstream retains `m_gpu` for the complete buffer lifetime. The Metal
    // buffers also refer back to their device, but keeping this explicit owner
    // preserves the pinned field-level ownership contract.
    _device: Retained<ProtocolObject<dyn MTLDevice>>,
    buffers: [Option<Retained<ProtocolObject<dyn MTLBuffer>>>; BUFFER_RING_SIZE],
    back_buffer_idx: Cell<usize>,
    submitted_buffer_idx: Cell<Option<usize>>,
    mapped_buffer_idx: Option<usize>,
    dirty: Cell<bool>,
    map_count: usize,
}

impl NativeMetalBuffer {
    /// Allocate one shared buffer for immutable-at-initialization data or the
    /// complete three-buffer ring for data that can be updated each frame.
    pub(crate) fn new(
        device: &ProtocolObject<dyn MTLDevice>,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Result<Self, RendererError> {
        let device_pointer = std::ptr::from_ref(device).cast_mut();
        // SAFETY: `device_pointer` comes directly from a live Objective-C
        // reference. Retaining it detaches the device lifetime from the
        // constructor borrow, matching upstream's strong `m_gpu` field.
        let retained_device = unsafe { Retained::retain(device_pointer) }
            .expect("a pointer derived from a reference is non-null");
        let buffer_count = buffer_count_for_flags(flags);
        let mut buffers = std::array::from_fn(|_| None);
        for slot in buffers.iter_mut().take(buffer_count) {
            *slot = Some(
                device
                    .newBufferWithLength_options(
                        size_in_bytes,
                        MTLResourceOptions::StorageModeShared,
                    )
                    .ok_or_else(|| {
                        RendererError::NativeMetal(format!(
                            "failed to allocate shared render buffer ({size_in_bytes} bytes)"
                        ))
                    })?,
            );
        }

        Ok(Self {
            buffer_type,
            flags,
            size_in_bytes,
            _device: retained_device,
            buffers,
            back_buffer_idx: Cell::new(0),
            submitted_buffer_idx: Cell::new(None),
            mapped_buffer_idx: None,
            dirty: Cell::new(false),
            map_count: 0,
        })
    }

    /// Return the front buffer selected by the most recent submitted update.
    /// Upstream starts with a front index of `-1`, so this is `None` until the
    /// first completed map is submitted. Consuming a dirty update advances the
    /// three-buffer ring here, not during `unmap()`, matching
    /// `RiveRenderBuffer::frontBufferIdx()`.
    pub(crate) fn submitted_buffer(&self) -> Option<&ProtocolObject<dyn MTLBuffer>> {
        assert!(
            self.mapped_buffer_idx.is_none(),
            "cannot submit a native Metal buffer while it is mapped"
        );
        if self.dirty.replace(false) {
            let submitted_index = self.back_buffer_idx.get();
            self.submitted_buffer_idx.set(Some(submitted_index));
            self.back_buffer_idx
                .set((submitted_index + 1) % BUFFER_RING_SIZE);
        }
        self.submitted_buffer_idx
            .get()
            .and_then(|index| self.buffers[index].as_deref())
    }

    /// Return the current back buffer for leaf-level integrations and tests.
    pub(crate) fn back_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        self.buffers[self.back_buffer_idx.get()]
            .as_deref()
            .expect("back buffer was allocated by NativeMetalBuffer::new")
    }

    fn map_back(&mut self) -> &mut [u8] {
        assert!(
            self.mapped_buffer_idx.is_none(),
            "native Metal buffer is already mapped"
        );
        assert!(
            self.flags != RenderBufferFlags::MappedOnceAtInitialization || self.map_count == 0,
            "mapped-once native Metal buffer cannot be mapped again"
        );
        let index = self.back_buffer_idx.get();
        self.mapped_buffer_idx = Some(index);
        self.dirty.set(true);
        self.map_count += 1;
        let buffer = self.buffers[index]
            .as_deref()
            .expect("back buffer was allocated by NativeMetalBuffer::new");
        let pointer: NonNull<c_void> = buffer.contents();
        // SAFETY: Metal returns a CPU-visible pointer for a buffer allocated
        // with MTLResourceStorageModeShared. The exact requested length was
        // used for the allocation and the buffer remains retained by `self`.
        unsafe { slice::from_raw_parts_mut(pointer.as_ptr().cast(), self.size_in_bytes) }
    }

    fn finish_unmap(&mut self) {
        assert!(
            self.mapped_buffer_idx.take().is_some(),
            "native Metal buffer is not mapped"
        );
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
        self.size_in_bytes
    }

    fn map_mut(&mut self) -> &mut [u8] {
        self.map_back()
    }

    fn unmap(&mut self) {
        // The upstream Metal `onUnmap()` is empty. Shared storage stays
        // CPU-visible; submission later consumes the base ring's dirty state.
        self.finish_unmap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct RingState {
        back: usize,
        submitted: Option<usize>,
        dirty: bool,
    }

    fn mark_dirty(mut state: RingState) -> RingState {
        state.dirty = true;
        state
    }

    fn submit(mut state: RingState) -> RingState {
        if state.dirty {
            state.submitted = Some(state.back);
            state.back = (state.back + 1) % BUFFER_RING_SIZE;
            state.dirty = false;
        }
        state
    }

    #[test]
    fn mapped_once_allocates_one_buffer_and_first_submit_uses_it() {
        assert_eq!(
            buffer_count_for_flags(RenderBufferFlags::MappedOnceAtInitialization),
            1
        );
        let state = submit(mark_dirty(RingState {
            back: 0,
            submitted: None,
            dirty: false,
        }));
        assert_eq!(
            state,
            RingState {
                back: 1,
                submitted: Some(0),
                dirty: false,
            }
        );
    }

    #[test]
    fn dirty_ring_rotates_only_when_submitted() {
        assert_eq!(
            buffer_count_for_flags(RenderBufferFlags::None),
            BUFFER_RING_SIZE
        );
        let mut state = RingState {
            back: 0,
            submitted: None,
            dirty: false,
        };
        for expected_front in 0..12 {
            state = mark_dirty(state);
            assert_eq!(state.back, expected_front % BUFFER_RING_SIZE);
            state = submit(state);
            assert_eq!(state.submitted, Some(expected_front % BUFFER_RING_SIZE));
            assert_eq!(state.back, (expected_front + 1) % BUFFER_RING_SIZE);
            assert!(!state.dirty);
        }
    }

    #[test]
    fn multiple_updates_before_submission_keep_the_same_back_buffer() {
        let state = RingState {
            back: 0,
            submitted: None,
            dirty: false,
        };
        let first_update = mark_dirty(state);
        let second_update = mark_dirty(first_update);
        assert_eq!(second_update.back, 0);
        assert_eq!(second_update.submitted, None);
        assert_eq!(submit(second_update).submitted, Some(0));
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_shared_buffer_mapping_and_ring_identity() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let mut mutable = NativeMetalBuffer::new(
            &device,
            RenderBufferType::Vertex,
            RenderBufferFlags::None,
            16,
        )
        .unwrap();
        let first_back = mutable.back_buffer() as *const ProtocolObject<dyn MTLBuffer>;
        mutable.map_mut()[..4].copy_from_slice(&[1, 2, 3, 4]);
        mutable.unmap();
        assert!(std::ptr::eq(
            first_back,
            mutable.back_buffer() as *const ProtocolObject<dyn MTLBuffer>
        ));
        mutable.map_mut()[..4].copy_from_slice(&[4, 3, 2, 1]);
        mutable.unmap();
        assert!(std::ptr::eq(
            first_back,
            mutable.back_buffer() as *const ProtocolObject<dyn MTLBuffer>
        ));
        let first_submitted = mutable.submitted_buffer().unwrap() as *const _;
        assert!(std::ptr::eq(first_back, first_submitted));
        let second_back = mutable.back_buffer() as *const ProtocolObject<dyn MTLBuffer>;
        assert!(!std::ptr::eq(first_back, second_back));

        let mut once = NativeMetalBuffer::new(
            &device,
            RenderBufferType::Index,
            RenderBufferFlags::MappedOnceAtInitialization,
            4,
        )
        .unwrap();
        let once_back = once.back_buffer() as *const ProtocolObject<dyn MTLBuffer>;
        once.map_mut().copy_from_slice(&[9, 8, 7, 6]);
        once.unmap();
        assert!(std::ptr::eq(
            once_back,
            once.submitted_buffer().unwrap() as *const _
        ));
        assert!(std::ptr::eq(
            once_back,
            once.submitted_buffer().unwrap() as *const _
        ));
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_map_unmap_protocol_rejects_invalid_sequences() {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let make_mutable = || {
            NativeMetalBuffer::new(
                &device,
                RenderBufferType::Vertex,
                RenderBufferFlags::None,
                4,
            )
            .unwrap()
        };

        let mut unmapped = make_mutable();
        assert!(catch_unwind(AssertUnwindSafe(|| unmapped.unmap())).is_err());

        let mut double_unmap = make_mutable();
        let _ = double_unmap.map_mut();
        double_unmap.unmap();
        assert!(catch_unwind(AssertUnwindSafe(|| double_unmap.unmap())).is_err());

        let mut nested_map = make_mutable();
        let _ = nested_map.map_mut();
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _ = nested_map.map_mut();
        }))
        .is_err());
        nested_map.unmap();

        let mut submit_while_mapped = make_mutable();
        let _ = submit_while_mapped.map_mut();
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _ = submit_while_mapped.submitted_buffer();
        }))
        .is_err());
        submit_while_mapped.unmap();

        let mut once = NativeMetalBuffer::new(
            &device,
            RenderBufferType::Index,
            RenderBufferFlags::MappedOnceAtInitialization,
            4,
        )
        .unwrap();
        let _ = once.map_mut();
        once.unmap();
        let _ = once.submitted_buffer();
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _ = once.map_mut();
        }))
        .is_err());
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_buffer_retains_the_upstream_device_owner() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let device_pointer = device.as_ref() as *const ProtocolObject<dyn MTLDevice>;
        let buffer = NativeMetalBuffer::new(
            &device,
            RenderBufferType::Vertex,
            RenderBufferFlags::None,
            4,
        )
        .unwrap();
        drop(device);
        assert!(std::ptr::eq(
            device_pointer,
            buffer._device.as_ref() as *const ProtocolObject<dyn MTLDevice>
        ));
    }
}
