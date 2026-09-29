//! Mechanical translation of
//! `renderer/rive_vk_bootstrap/include/rive_vk_bootstrap/vulkan_frame_sync_coordinator.hpp`
//! and `tests/unit_tests/renderer/vulkan_frame_sync_coordinator_test.cpp`.
//! Pinned upstream: `d619bc2a83f3c592a57eb58b9c83315142bcfbfc`.

#![allow(non_snake_case)]

use std::collections::VecDeque;

// The source substitutes its test synchronizer before including the header.
// This surface permits that same substitution without a Vulkan device.
pub(crate) trait FrameSynchronizer {
    fn current_frame_number(&self) -> u64;
    fn safe_frame_number(&self) -> u64;
    fn check_most_recent_frame_completion(&self) -> bool;
}

struct FramePair {
    synchronizerFrame: u64,
    coordinatedFrame: u64,
}

struct SynchronizerData<'a, S: FrameSynchronizer> {
    synchronizer: &'a S,
    frames: VecDeque<FramePair>,
}

pub(crate) struct VulkanFrameSyncCoordinator<'a, S: FrameSynchronizer> {
    m_synchronizers: Vec<SynchronizerData<'a, S>>,
    m_currentCoordinatedFrame: u64,
    m_coordinatedSafeFrame: u64,
}

impl<'a, S: FrameSynchronizer> Default for VulkanFrameSyncCoordinator<'a, S> {
    fn default() -> Self {
        Self {
            m_synchronizers: Vec::new(),
            m_currentCoordinatedFrame: 0,
            m_coordinatedSafeFrame: 0,
        }
    }
}

impl<'a, S: FrameSynchronizer> VulkanFrameSyncCoordinator<'a, S> {
    fn findSyncData(&self, s: &S) -> Option<usize> {
        self.m_synchronizers
            .iter()
            .position(|d| std::ptr::eq(d.synchronizer, s))
    }

    pub(crate) fn addFrameSynchronizer(&mut self, sync: &'a S) {
        // A Rust reference supplies the source's non-null precondition.
        debug_assert!(self.findSyncData(sync).is_none());
        self.m_synchronizers.push(SynchronizerData {
            synchronizer: sync,
            frames: VecDeque::new(),
        });
    }

    pub(crate) fn removeFrameSynchronizer(&mut self, sync: &S) {
        let found = self.findSyncData(sync);
        debug_assert!(found.is_some());
        self.m_synchronizers.remove(found.unwrap());
    }

    pub(crate) fn currentFrameNumber(&self) -> u64 {
        self.m_currentCoordinatedFrame
    }

    pub(crate) fn safeFrameNumber(&self) -> u64 {
        self.m_coordinatedSafeFrame
    }

    pub(crate) fn onFrameStart(&mut self, synchronizer: &S) {
        let synchronizerCurrentFrame = synchronizer.current_frame_number();
        let synchronizerSafeFrame = synchronizer.safe_frame_number();

        // Preserve the source's linear search and non-owning pointer identity.
        let found = self.findSyncData(synchronizer);
        debug_assert!(found.is_some());
        let found = &mut self.m_synchronizers[found.unwrap()];

        debug_assert!(
            found.frames.is_empty()
                || found.frames.back().unwrap().synchronizerFrame < synchronizerCurrentFrame
        );
        if found.frames.is_empty() && synchronizerSafeFrame != synchronizerCurrentFrame {
            // Seed every outstanding frame at the previous coordinated frame.
            for frame in synchronizerSafeFrame..synchronizerCurrentFrame {
                found.frames.push_back(FramePair {
                    synchronizerFrame: frame,
                    coordinatedFrame: self.m_currentCoordinatedFrame,
                });
            }
        } else {
            while !found.frames.is_empty()
                && found.frames.front().unwrap().synchronizerFrame < synchronizerSafeFrame
            {
                found.frames.pop_front();
            }
        }

        self.m_currentCoordinatedFrame = self.m_currentCoordinatedFrame.wrapping_add(1);
        found.frames.push_back(FramePair {
            synchronizerFrame: synchronizerCurrentFrame,
            coordinatedFrame: self.m_currentCoordinatedFrame,
        });
        self.m_coordinatedSafeFrame = found.frames.front().unwrap().coordinatedFrame;

        for s in &mut self.m_synchronizers {
            if s.frames.is_empty()
                || s.frames.front().unwrap().coordinatedFrame >= self.m_coordinatedSafeFrame
            {
                continue;
            }
            if s.synchronizer.check_most_recent_frame_completion() {
                s.frames.clear();
            } else {
                self.m_coordinatedSafeFrame = s.frames.front().unwrap().coordinatedFrame;
            }
        }

        debug_assert!(self.m_currentCoordinatedFrame > self.m_coordinatedSafeFrame);
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameSynchronizer, VulkanFrameSyncCoordinator};
    use std::cell::Cell;

    #[derive(Default)]
    struct VulkanFrameSynchronizer {
        m_isMostRecentFrameDone: Cell<bool>,
        m_currentFrameNumber: Cell<u64>,
        m_safeFrameNumber: Cell<u64>,
    }

    impl FrameSynchronizer for VulkanFrameSynchronizer {
        fn check_most_recent_frame_completion(&self) -> bool {
            self.m_isMostRecentFrameDone.get()
        }
        fn current_frame_number(&self) -> u64 {
            self.m_currentFrameNumber.get()
        }
        fn safe_frame_number(&self) -> u64 {
            self.m_safeFrameNumber.get()
        }
    }

    impl VulkanFrameSynchronizer {
        fn tickFrameAndSafe(&self) {
            self.m_isMostRecentFrameDone.set(false);
            self.m_currentFrameNumber
                .set(self.m_currentFrameNumber.get().wrapping_add(1));
            self.m_safeFrameNumber
                .set(self.m_safeFrameNumber.get().wrapping_add(1));
        }
    }

    // TEST_CASE("Single Frame Sync", "[vulkan_frame_sync_coordinator]")
    #[test]
    fn single_frame_sync() {
        let sync = VulkanFrameSynchronizer::default();
        let mut coordinator = VulkanFrameSyncCoordinator::default();
        sync.m_currentFrameNumber.set(2);
        coordinator.addFrameSynchronizer(&sync);

        sync.tickFrameAndSafe();
        coordinator.onFrameStart(&sync);
        assert_eq!(coordinator.currentFrameNumber(), 1);
        // A new synchronizer cannot make the current frame safe.
        assert_eq!(coordinator.safeFrameNumber(), 0);

        sync.tickFrameAndSafe();
        coordinator.onFrameStart(&sync);
        assert_eq!(coordinator.currentFrameNumber(), 2);
        assert_eq!(coordinator.safeFrameNumber(), 0);

        for f in 3..20 {
            sync.tickFrameAndSafe();
            coordinator.onFrameStart(&sync);
            assert_eq!(coordinator.currentFrameNumber(), f);
            assert_eq!(coordinator.safeFrameNumber(), f - 2);
        }
    }

    // TEST_CASE("Replaced Frame Sync", "[vulkan_frame_sync_coordinator]")
    #[test]
    fn replaced_frame_sync() {
        let syncA = VulkanFrameSynchronizer::default();
        syncA.m_currentFrameNumber.set(1);
        let syncB = VulkanFrameSynchronizer::default();
        let mut coordinator = VulkanFrameSyncCoordinator::default();
        coordinator.addFrameSynchronizer(&syncA);

        syncA.tickFrameAndSafe();
        coordinator.onFrameStart(&syncA);
        assert_eq!(coordinator.currentFrameNumber(), 1);
        assert_eq!(coordinator.safeFrameNumber(), 0);

        syncA.tickFrameAndSafe();
        coordinator.onFrameStart(&syncA);
        assert_eq!(coordinator.currentFrameNumber(), 2);
        assert_eq!(coordinator.safeFrameNumber(), 1);

        // Arbitrarily different frame numbers, as in the source.
        syncB.m_currentFrameNumber.set(1000);
        syncB.m_safeFrameNumber.set(999);
        coordinator.removeFrameSynchronizer(&syncA);
        coordinator.addFrameSynchronizer(&syncB);

        coordinator.onFrameStart(&syncB);
        assert_eq!(coordinator.currentFrameNumber(), 3);
        assert_eq!(coordinator.safeFrameNumber(), 2);

        syncB.tickFrameAndSafe();
        coordinator.onFrameStart(&syncB);
        assert_eq!(coordinator.currentFrameNumber(), 4);
        assert_eq!(coordinator.safeFrameNumber(), 3);

        syncB.tickFrameAndSafe();
        coordinator.onFrameStart(&syncB);
        assert_eq!(coordinator.currentFrameNumber(), 5);
        assert_eq!(coordinator.safeFrameNumber(), 4);
    }

    // TEST_CASE("Two Frame Syncs", "[vulkan_frame_sync_coordinator]")
    #[test]
    fn two_frame_syncs() {
        let syncA = VulkanFrameSynchronizer::default();
        let syncB = VulkanFrameSynchronizer::default();
        let mut coordinator = VulkanFrameSyncCoordinator::default();
        coordinator.addFrameSynchronizer(&syncA);
        coordinator.addFrameSynchronizer(&syncB);

        syncA.m_currentFrameNumber.set(1000);
        syncA
            .m_safeFrameNumber
            .set(syncA.m_currentFrameNumber.get() - 2);
        syncB.m_currentFrameNumber.set(500000);
        syncB
            .m_safeFrameNumber
            .set(syncB.m_currentFrameNumber.get() - 1);

        // Coordinated frame 1 is A's frame 1001, safe at 999.
        syncA.tickFrameAndSafe();
        coordinator.onFrameStart(&syncA);
        assert_eq!(coordinator.currentFrameNumber(), 1);
        assert_eq!(coordinator.safeFrameNumber(), 0);

        // A holds back the safe frame while B advances.
        for f in 2..=5 {
            syncB.tickFrameAndSafe();
            coordinator.onFrameStart(&syncB);
            assert_eq!(coordinator.currentFrameNumber(), f);
            assert_eq!(coordinator.safeFrameNumber(), 0);
        }

        for f in 6..=7 {
            syncA.tickFrameAndSafe();
            coordinator.onFrameStart(&syncA);
            assert_eq!(coordinator.currentFrameNumber(), f);
            assert_eq!(coordinator.safeFrameNumber(), f - 6);
        }

        // A moves past B, so B's safe frame takes over.
        syncA.tickFrameAndSafe();
        coordinator.onFrameStart(&syncA);
        assert_eq!(coordinator.currentFrameNumber(), 8);
        assert_eq!(coordinator.safeFrameNumber(), 4);

        // B finishes its most recent frame, leaving A as the deciding factor.
        syncB.m_isMostRecentFrameDone.set(true);
        syncA.tickFrameAndSafe();
        coordinator.onFrameStart(&syncA);
        assert_eq!(coordinator.currentFrameNumber(), 9);
        assert_eq!(coordinator.safeFrameNumber(), 7);

        for f in 10..=15 {
            syncB.tickFrameAndSafe();
            coordinator.onFrameStart(&syncB);
            assert_eq!(coordinator.currentFrameNumber(), f);
            assert_eq!(coordinator.safeFrameNumber(), 7);
        }

        syncA.tickFrameAndSafe();
        coordinator.onFrameStart(&syncA);
        assert_eq!(coordinator.currentFrameNumber(), 16);
        assert_eq!(coordinator.safeFrameNumber(), 8);

        syncA.tickFrameAndSafe();
        coordinator.onFrameStart(&syncA);
        assert_eq!(coordinator.currentFrameNumber(), 17);
        assert_eq!(coordinator.safeFrameNumber(), 9);

        // The third tick passes B's safe frame, which becomes the minimum.
        syncA.tickFrameAndSafe();
        coordinator.onFrameStart(&syncA);
        assert_eq!(coordinator.currentFrameNumber(), 18);
        assert_eq!(coordinator.safeFrameNumber(), 14);

        coordinator.removeFrameSynchronizer(&syncB);
        syncA.tickFrameAndSafe();
        coordinator.onFrameStart(&syncA);
        assert_eq!(coordinator.currentFrameNumber(), 19);
        assert_eq!(coordinator.safeFrameNumber(), 17);

        // Re-adding B without ticking it does not affect the minimum.
        coordinator.addFrameSynchronizer(&syncB);
        for f in 20..=25 {
            syncA.tickFrameAndSafe();
            coordinator.onFrameStart(&syncA);
            assert_eq!(coordinator.currentFrameNumber(), f);
            assert_eq!(coordinator.safeFrameNumber(), f - 2);
        }

        coordinator.removeFrameSynchronizer(&syncA);
        syncB.tickFrameAndSafe();
        coordinator.onFrameStart(&syncB);
        assert_eq!(coordinator.currentFrameNumber(), 26);
        assert_eq!(coordinator.safeFrameNumber(), 25);
    }

    // TEST_CASE("Resumed synchronizer", "[vulkan_frame_sync_coordinator]")
    #[test]
    fn resumed_synchronizer() {
        let syncA = VulkanFrameSynchronizer::default();
        // Storage outlives the coordinator's borrow; registration remains
        // scoped exactly like the source's temporary synchronizer below.
        let syncB = VulkanFrameSynchronizer::default();
        let mut coordinator = VulkanFrameSyncCoordinator::default();
        coordinator.addFrameSynchronizer(&syncA);

        syncA.m_currentFrameNumber.set(2);
        coordinator.onFrameStart(&syncA);
        syncA.m_isMostRecentFrameDone.set(true);

        {
            coordinator.addFrameSynchronizer(&syncB);
            syncB.m_currentFrameNumber.set(1);
            coordinator.onFrameStart(&syncB);
            assert_eq!(coordinator.safeFrameNumber(), 1);
            coordinator.removeFrameSynchronizer(&syncB);
        }

        syncA.tickFrameAndSafe();
        coordinator.onFrameStart(&syncA);
        assert_eq!(coordinator.safeFrameNumber(), 2);

        syncA.tickFrameAndSafe();
        coordinator.onFrameStart(&syncA);
        assert_eq!(coordinator.safeFrameNumber(), 2);

        syncA.tickFrameAndSafe();
        coordinator.onFrameStart(&syncA);
        assert_eq!(coordinator.safeFrameNumber(), 3);
    }
}
