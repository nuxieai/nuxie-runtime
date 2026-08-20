//! Native Metal resource-ring policy translated from the pinned upstream
//! `renderer/include/rive/renderer/buffer_ring.hpp:22-80`,
//! `renderer/include/rive/renderer/metal/render_context_metal_impl.h:208-212,277-280`,
//! and `renderer/src/metal/render_context_metal_impl.mm:414-452,1251-1264,2016-2030`.
//!
//! Pinned upstream source: `rive-runtime` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
//!
//! The Metal context starts with ring index zero and advances it before each
//! flush.  The first three flushes therefore select slots 1, 2, and 0.  A
//! selected slot remains unavailable until the command buffer completion path
//! releases it; an abandoned submission releases it immediately.

/// The number of independently reservable flush slots in the upstream ring.
pub(crate) const RESOURCE_RING_SIZE: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResourceRingError {
    InvalidSlot,
    SlotInFlight,
    SlotNotInFlight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotState {
    Available,
    InFlight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResourceRingSlot {
    state: SlotState,
    /// Number of successful release/abandon operations for this slot.  This
    /// is deliberately part of the policy object so tests can prove that a
    /// completion and an abandonment cannot release the same slot twice.
    release_count: u32,
}

impl ResourceRingSlot {
    const AVAILABLE: Self = Self {
        state: SlotState::Available,
        release_count: 0,
    };
}

/// Concrete three-slot policy for transient native Metal resources.
///
/// This is intentionally not generic over a backend resource.  The future
/// Metal adapter can pair the returned index with its concrete `MTLBuffer` or
/// other resource while this type keeps the ownership/lifetime policy pure and
/// testable.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ResourceRing {
    slots: [ResourceRingSlot; RESOURCE_RING_SIZE],
    next_flush_index: usize,
}

impl ResourceRing {
    /// Create the three-slot submission policy.
    ///
    /// This type deliberately does not model byte capacity or alignment.
    /// Upstream preserves each `BufferRingMetalImpl` capacity verbatim and
    /// applies 256-byte alignment to the buffer elements/offsets that require
    /// it, not to this submission-lock rotation.
    pub(crate) fn new() -> Self {
        Self {
            slots: [ResourceRingSlot::AVAILABLE; RESOURCE_RING_SIZE],
            // Upstream `m_bufferRingIdx` is initialized to zero and advances
            // before locking in `prepareToFlush`, making slot 1 the first one.
            next_flush_index: 0,
        }
    }

    /// Select and reserve the next slot for a flush.
    ///
    /// This mirrors the upstream `(index + 1) % kBufferRingSize` followed by
    /// locking.  Returning an error instead of blocking keeps this leaf pure;
    /// the platform command-buffer completion callback is responsible for
    /// calling [`Self::release`] when the GPU is done.
    pub(crate) fn prepare_to_flush(&mut self) -> Result<usize, ResourceRingError> {
        let index = (self.next_flush_index + 1) % RESOURCE_RING_SIZE;
        let slot = &mut self.slots[index];
        if slot.state == SlotState::InFlight {
            return Err(ResourceRingError::SlotInFlight);
        }
        self.next_flush_index = index;
        slot.state = SlotState::InFlight;
        Ok(index)
    }

    /// Release a slot after its command buffer completes on the GPU.
    pub(crate) fn release(&mut self, index: usize) -> Result<(), ResourceRingError> {
        self.finish(index)
    }

    /// Abandon an unsubmitted flush and release its slot immediately.
    pub(crate) fn abandon(&mut self, index: usize) -> Result<(), ResourceRingError> {
        self.finish(index)
    }

    pub(crate) fn is_in_flight(&self, index: usize) -> Result<bool, ResourceRingError> {
        self.slots
            .get(index)
            .map(|slot| slot.state == SlotState::InFlight)
            .ok_or(ResourceRingError::InvalidSlot)
    }

    #[cfg(test)]
    fn release_count(&self, index: usize) -> u32 {
        self.slots[index].release_count
    }

    fn finish(&mut self, index: usize) -> Result<(), ResourceRingError> {
        let slot = self
            .slots
            .get_mut(index)
            .ok_or(ResourceRingError::InvalidSlot)?;
        if slot.state != SlotState::InFlight {
            return Err(ResourceRingError::SlotNotInFlight);
        }
        slot.state = SlotState::Available;
        slot.release_count += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_to_flush_rotates_one_two_zero() {
        let mut ring = ResourceRing::new();
        let first = ring.prepare_to_flush().unwrap();
        ring.release(first).unwrap();
        let second = ring.prepare_to_flush().unwrap();
        ring.release(second).unwrap();
        let third = ring.prepare_to_flush().unwrap();
        ring.release(third).unwrap();
        assert_eq!([first, second, third], [1, 2, 0]);
    }

    #[test]
    fn an_in_flight_slot_cannot_be_reused() {
        let mut ring = ResourceRing::new();
        let first = ring.prepare_to_flush().unwrap();
        let second = ring.prepare_to_flush().unwrap();
        let third = ring.prepare_to_flush().unwrap();
        assert_eq!(
            ring.prepare_to_flush(),
            Err(ResourceRingError::SlotInFlight)
        );
        assert!(ring.is_in_flight(first).unwrap());
        assert!(ring.is_in_flight(second).unwrap());
        assert!(ring.is_in_flight(third).unwrap());
        ring.release(first).unwrap();
        let reused = ring.prepare_to_flush().unwrap();
        assert_eq!(reused, first);
    }

    #[test]
    fn completion_releases_exactly_once() {
        let mut ring = ResourceRing::new();
        let slot = ring.prepare_to_flush().unwrap();
        ring.release(slot).unwrap();
        assert_eq!(ring.release_count(slot), 1);
        assert_eq!(ring.release(slot), Err(ResourceRingError::SlotNotInFlight));
        assert_eq!(ring.abandon(slot), Err(ResourceRingError::SlotNotInFlight));
        assert_eq!(ring.release_count(slot), 1);
    }

    #[test]
    fn abandonment_releases_exactly_once() {
        let mut ring = ResourceRing::new();
        let slot = ring.prepare_to_flush().unwrap();
        ring.abandon(slot).unwrap();
        assert_eq!(ring.release_count(slot), 1);
        assert_eq!(ring.abandon(slot), Err(ResourceRingError::SlotNotInFlight));
        assert_eq!(ring.release(slot), Err(ResourceRingError::SlotNotInFlight));
        assert_eq!(ring.release_count(slot), 1);
    }

    #[test]
    fn invalid_slots_never_change_ring_state() {
        let mut ring = ResourceRing::new();
        assert_eq!(
            ring.release(RESOURCE_RING_SIZE),
            Err(ResourceRingError::InvalidSlot)
        );
        assert_eq!(
            ring.abandon(RESOURCE_RING_SIZE),
            Err(ResourceRingError::InvalidSlot)
        );
        assert_eq!(
            ring.is_in_flight(RESOURCE_RING_SIZE),
            Err(ResourceRingError::InvalidSlot)
        );
        assert_eq!(ring.prepare_to_flush().unwrap(), 1);
    }
}
