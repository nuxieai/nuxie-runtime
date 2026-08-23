//! Blocking ownership coordinator for the native Metal buffer rings.
//!
//! Ported from pinned upstream `rive-runtime` commit
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`:
//! - `renderer/include/rive/renderer/metal/render_context_metal_impl.h:208-212`
//! - `renderer/include/rive/renderer/metal/render_context_metal_impl.h:277-280`
//! - `renderer/src/metal/render_context_metal_impl.mm:1251-1264`
//! - `renderer/src/metal/render_context_metal_impl.mm:2016-2029`
//!
//! Upstream advances through three slots in the order 1, 2, 0 and blocks when
//! the next slot is still owned by the GPU. [`BufferRingLease`] represents the
//! CPU/pre-submit ownership interval. Submission transfers that ownership to a
//! [`BufferRingCompletion`], which the command-buffer completion path must
//! complete exactly once. Dropping an unsubmitted lease abandons it safely.
//! Dropping a submitted completion without completing it intentionally leaves
//! its slot unavailable: leaking capacity is safer than reusing GPU-owned
//! memory early.

use std::sync::{Arc, Condvar, Mutex, MutexGuard};

const BUFFER_RING_SIZE: usize = 3;

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
    slot_released: Condvar,
}

#[derive(Debug)]
struct State {
    current_slot: usize,
    next_reservation: u64,
    outstanding: [Option<u64>; BUFFER_RING_SIZE],
    prepare_in_progress: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BufferRingCoordinatorError {
    AlreadyCompleted,
    AlreadyTransferred,
    InvalidSlot,
    ReservationMismatch,
    ReservationNotOutstanding,
}

impl Shared {
    /// A poisoned coordinator mutex does not imply that Metal has finished
    /// with any ring slot. Recover the last state and continue enforcing its
    /// outstanding reservations. This deliberately fails closed: uncertainty
    /// can leak a slot, but can never make an in-flight slot reusable early.
    fn lock_state(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn wait_for_slot<'a>(&self, state: MutexGuard<'a, State>) -> MutexGuard<'a, State> {
        self.slot_released
            .wait(state)
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn release(&self, slot: usize, reservation: u64) -> Result<(), BufferRingCoordinatorError> {
        let mut state = self.lock_state();
        let outstanding = state
            .outstanding
            .get_mut(slot)
            .ok_or(BufferRingCoordinatorError::InvalidSlot)?;
        match *outstanding {
            Some(active) if active == reservation => {
                *outstanding = None;
                self.slot_released.notify_all();
                Ok(())
            }
            Some(_) => Err(BufferRingCoordinatorError::ReservationMismatch),
            None => Err(BufferRingCoordinatorError::ReservationNotOutstanding),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BufferRingCoordinator {
    shared: Arc<Shared>,
}

impl BufferRingCoordinator {
    /// Creates the coordinator at upstream's initial slot-zero cursor.
    pub(crate) fn new() -> Self {
        Self {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    current_slot: 0,
                    next_reservation: 1,
                    outstanding: [None; BUFFER_RING_SIZE],
                    prepare_in_progress: false,
                }),
                slot_released: Condvar::new(),
            }),
        }
    }

    /// Reserves the next ring slot, blocking until Metal has completed its
    /// previous use of that slot.
    pub(crate) fn prepare_to_flush(&self) -> BufferRingLease {
        let mut state = self.shared.lock_state();

        // Upstream has one render-thread producer. Serializing prepare calls
        // preserves that ordering if this Rust owner is accidentally shared
        // by more than one producer while a ring slot is blocked.
        while state.prepare_in_progress {
            state = self.shared.wait_for_slot(state);
        }
        state.prepare_in_progress = true;

        let slot = (state.current_slot + 1) % BUFFER_RING_SIZE;
        while state.outstanding[slot].is_some() {
            state = self.shared.wait_for_slot(state);
        }

        let reservation = state.next_reservation;
        state.next_reservation = state.next_reservation.wrapping_add(1).max(1);
        state.current_slot = slot;
        state.outstanding[slot] = Some(reservation);
        state.prepare_in_progress = false;
        self.shared.slot_released.notify_all();

        BufferRingLease {
            shared: Arc::clone(&self.shared),
            slot,
            reservation,
            transferred: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn slot_is_available_for_test(&self, slot: usize) -> bool {
        self.shared
            .lock_state()
            .outstanding
            .get(slot)
            .is_some_and(Option::is_none)
    }
}

#[derive(Debug)]
pub(crate) struct BufferRingLease {
    shared: Arc<Shared>,
    slot: usize,
    reservation: u64,
    transferred: bool,
}

impl BufferRingLease {
    /// Returns the ring index whose concrete buffers may be modified.
    #[cfg(test)]
    pub(crate) fn slot(&self) -> usize {
        self.slot
    }

    /// Hands release responsibility to the command-buffer completion path.
    ///
    /// After this succeeds, dropping this pre-submit lease does not release
    /// the slot. The returned completion owner must live until the Metal
    /// completion path invokes [`BufferRingCompletion::complete`].
    pub(crate) fn transfer_to_completion(
        &mut self,
    ) -> Result<BufferRingCompletion, BufferRingCoordinatorError> {
        if self.transferred {
            return Err(BufferRingCoordinatorError::AlreadyTransferred);
        }
        self.transferred = true;
        Ok(BufferRingCompletion {
            shared: Arc::clone(&self.shared),
            slot: self.slot,
            reservation: self.reservation,
            completed: false,
        })
    }
}

impl Drop for BufferRingLease {
    fn drop(&mut self) {
        if !self.transferred {
            // A mismatch indicates an internal ownership bug. Keep the slot
            // unavailable rather than risking reuse while Metal may own it.
            let _ = self.shared.release(self.slot, self.reservation);
        }
    }
}

#[derive(Debug)]
pub(crate) struct BufferRingCompletion {
    shared: Arc<Shared>,
    slot: usize,
    reservation: u64,
    completed: bool,
}

impl BufferRingCompletion {
    /// Releases the submitted ring slot exactly once and wakes a waiter.
    pub(crate) fn complete(&mut self) -> Result<(), BufferRingCoordinatorError> {
        if self.completed {
            return Err(BufferRingCoordinatorError::AlreadyCompleted);
        }
        self.shared.release(self.slot, self.reservation)?;
        self.completed = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    const SHORT_WAIT: Duration = Duration::from_millis(50);
    const LONG_WAIT: Duration = Duration::from_secs(2);

    #[test]
    fn reservations_begin_with_slots_one_two_zero() {
        let coordinator = BufferRingCoordinator::new();

        let first = coordinator.prepare_to_flush();
        let second = coordinator.prepare_to_flush();
        let third = coordinator.prepare_to_flush();

        assert_eq!([first.slot(), second.slot(), third.slot()], [1, 2, 0]);
    }

    #[test]
    fn fourth_reservation_waits_for_submitted_completion() {
        let coordinator = BufferRingCoordinator::new();
        let mut first = coordinator.prepare_to_flush();
        let _second = coordinator.prepare_to_flush();
        let _third = coordinator.prepare_to_flush();
        let mut completion = first.transfer_to_completion().unwrap();

        let (started_tx, started_rx) = mpsc::channel();
        let (reserved_tx, reserved_rx) = mpsc::channel();
        let waiter = coordinator.clone();
        let thread = thread::spawn(move || {
            started_tx.send(()).unwrap();
            let lease = waiter.prepare_to_flush();
            reserved_tx.send(lease.slot()).unwrap();
        });

        started_rx.recv_timeout(LONG_WAIT).unwrap();
        assert_eq!(
            reserved_rx.recv_timeout(SHORT_WAIT),
            Err(mpsc::RecvTimeoutError::Timeout)
        );

        // Submission transferred the slot to the completion owner. Dropping
        // the pre-submit lease must therefore leave the waiter blocked.
        drop(first);
        assert_eq!(
            reserved_rx.recv_timeout(SHORT_WAIT),
            Err(mpsc::RecvTimeoutError::Timeout)
        );

        completion.complete().unwrap();
        assert_eq!(reserved_rx.recv_timeout(LONG_WAIT).unwrap(), 1);
        thread.join().unwrap();
    }

    #[test]
    fn abandoning_an_unsubmitted_lease_unblocks_the_next_reservation() {
        let coordinator = BufferRingCoordinator::new();
        let first = coordinator.prepare_to_flush();
        let _second = coordinator.prepare_to_flush();
        let _third = coordinator.prepare_to_flush();

        let (started_tx, started_rx) = mpsc::channel();
        let (reserved_tx, reserved_rx) = mpsc::channel();
        let waiter = coordinator.clone();
        let thread = thread::spawn(move || {
            started_tx.send(()).unwrap();
            let lease = waiter.prepare_to_flush();
            reserved_tx.send(lease.slot()).unwrap();
        });

        started_rx.recv_timeout(LONG_WAIT).unwrap();
        assert_eq!(
            reserved_rx.recv_timeout(SHORT_WAIT),
            Err(mpsc::RecvTimeoutError::Timeout)
        );

        drop(first);
        assert_eq!(reserved_rx.recv_timeout(LONG_WAIT).unwrap(), 1);
        thread.join().unwrap();
    }

    #[test]
    fn repeated_abandonment_and_completion_cycles_do_not_leak_slots() {
        let coordinator = BufferRingCoordinator::new();

        for reservation in 0..300 {
            let mut lease = coordinator.prepare_to_flush();
            assert_eq!(lease.slot(), (reservation + 1) % BUFFER_RING_SIZE);
            if reservation % 2 == 0 {
                drop(lease);
            } else {
                let mut completion = lease.transfer_to_completion().unwrap();
                drop(lease);
                completion.complete().unwrap();
            }
        }
    }

    #[test]
    fn transfer_and_completion_are_each_accepted_exactly_once() {
        let coordinator = BufferRingCoordinator::new();
        let mut lease = coordinator.prepare_to_flush();
        let mut completion = lease.transfer_to_completion().unwrap();

        assert!(matches!(
            lease.transfer_to_completion(),
            Err(BufferRingCoordinatorError::AlreadyTransferred)
        ));
        drop(lease);

        assert_eq!(completion.complete(), Ok(()));
        assert_eq!(
            completion.complete(),
            Err(BufferRingCoordinatorError::AlreadyCompleted)
        );
    }

    #[test]
    fn invalid_or_mismatched_completions_cannot_release_a_slot() {
        let coordinator = BufferRingCoordinator::new();
        let lease = coordinator.prepare_to_flush();
        let reservation = lease.reservation;

        let mut invalid_slot = BufferRingCompletion {
            shared: Arc::clone(&coordinator.shared),
            slot: BUFFER_RING_SIZE,
            reservation,
            completed: false,
        };
        assert_eq!(
            invalid_slot.complete(),
            Err(BufferRingCoordinatorError::InvalidSlot)
        );

        let mut wrong_reservation = BufferRingCompletion {
            shared: Arc::clone(&coordinator.shared),
            slot: lease.slot(),
            reservation: reservation + 1,
            completed: false,
        };
        assert_eq!(
            wrong_reservation.complete(),
            Err(BufferRingCoordinatorError::ReservationMismatch)
        );

        drop(lease);
    }

    #[test]
    fn poison_recovery_preserves_outstanding_ownership() {
        let coordinator = BufferRingCoordinator::new();
        let first = coordinator.prepare_to_flush();
        let _second = coordinator.prepare_to_flush();
        let _third = coordinator.prepare_to_flush();

        let poison_target = coordinator.clone();
        assert!(thread::spawn(move || {
            let _state = poison_target.shared.state.lock().unwrap();
            panic!("poison the coordinator after reservations are recorded");
        })
        .join()
        .is_err());

        let (started_tx, started_rx) = mpsc::channel();
        let (reserved_tx, reserved_rx) = mpsc::channel();
        let waiter = coordinator.clone();
        let thread = thread::spawn(move || {
            started_tx.send(()).unwrap();
            let lease = waiter.prepare_to_flush();
            reserved_tx.send(lease.slot()).unwrap();
        });

        started_rx.recv_timeout(LONG_WAIT).unwrap();
        assert_eq!(
            reserved_rx.recv_timeout(SHORT_WAIT),
            Err(mpsc::RecvTimeoutError::Timeout)
        );

        drop(first);
        assert_eq!(reserved_rx.recv_timeout(LONG_WAIT).unwrap(), 1);
        thread.join().unwrap();
    }
}
