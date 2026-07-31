//! Direct Rust owner for pinned C++ `src/data_bind/data_bind_container.cpp`.
//!
//! The queue retains authored occurrence order, partitions target-to-source
//! work ahead of target work, rejects recursive entry, and defers membership
//! changes made during an active pass. Additions flush before removals, so a
//! same-tick add-then-remove resolves to removed exactly like C++
//! (`data_bind_container.cpp:54-112,156-225,245-267`).

use std::collections::BTreeSet;

use crate::retained_data_bind::RuntimeRetainedDataBind;
use crate::view_model_cell::RuntimeCellNotificationQueue;

#[derive(Debug, Default)]
pub(crate) struct RuntimeDataBindContainerQueue {
    pending: RuntimeCellNotificationQueue,
    reporting: Vec<usize>,
    active_dirty: BTreeSet<usize>,
    members: BTreeSet<usize>,
    persisting: Vec<usize>,
    pending_additions: Vec<(usize, bool)>,
    pending_removals: Vec<usize>,
    processing: bool,
    next_occurrence_index: usize,
}

impl RuntimeDataBindContainerQueue {
    /// Register one occurrence in the same append order as C++
    /// `DataBindContainer::addDataBind`.
    pub(crate) fn add_data_bind(
        &mut self,
        data_bind: &mut RuntimeRetainedDataBind,
        persisting: bool,
    ) -> usize {
        let index = self.next_occurrence_index;
        self.next_occurrence_index += 1;
        data_bind.report_source_dirt_to(&self.pending, index);
        if self.processing {
            self.pending_additions.push((index, persisting));
        } else {
            self.install_occurrence(index, persisting);
        }
        index
    }

    fn install_occurrence(&mut self, occurrence_index: usize, persisting: bool) {
        self.members.insert(occurrence_index);
        if persisting && !self.persisting.contains(&occurrence_index) {
            self.persisting.push(occurrence_index);
        }
    }

    /// Remove one occurrence from every queue it can inhabit.
    ///
    /// During processing the occurrence remains live until the active
    /// snapshots finish. This mirrors C++'s pointer/back-reference lifetime
    /// and avoids invalidating the active persisting/dirty iterators.
    pub(crate) fn remove_data_bind(&mut self, occurrence_index: usize) {
        if self.processing {
            self.pending_removals.push(occurrence_index);
            return;
        }
        self.uninstall_occurrence(occurrence_index);
    }

    fn uninstall_occurrence(&mut self, occurrence_index: usize) {
        self.members.remove(&occurrence_index);
        self.persisting
            .retain(|candidate| *candidate != occurrence_index);
        self.active_dirty.remove(&occurrence_index);
        self.pending.remove_data_bind(occurrence_index);
        self.reporting
            .retain(|candidate| *candidate != occurrence_index);
    }

    pub(crate) fn contains(&self, occurrence_index: usize) -> bool {
        self.members.contains(&occurrence_index)
    }

    pub(crate) fn is_persisting(&self, occurrence_index: usize) -> bool {
        self.persisting.contains(&occurrence_index)
    }

    pub(crate) fn has_pending_work(&self) -> bool {
        !self.persisting.is_empty() || !self.pending.is_empty()
    }

    /// Begin one non-recursive `updateDataBinds` pass.
    ///
    /// `None` means a recursive call was rejected. `Some(empty)` is a valid
    /// no-work pass and must still be paired with [`Self::finish_update`].
    pub(crate) fn begin_update(
        &mut self,
        mut is_to_source: impl FnMut(usize) -> bool,
    ) -> Option<Vec<usize>> {
        if self.processing {
            return None;
        }
        self.processing = true;
        self.pending.swap_into(&mut self.reporting);
        self.active_dirty.clear();

        // A retained sink normally coalesces while dirty. Re-homing an
        // already-dirty bind or folding its source sink can report the same
        // occurrence again before the snapshot, but C++ holds each pointer at
        // most once in a dirty list. Preserve only the first enqueue.
        let mut seen = BTreeSet::new();
        let mut to_source = self
            .persisting
            .iter()
            .copied()
            .filter(|occurrence| self.members.contains(occurrence))
            .collect::<Vec<_>>();
        seen.extend(to_source.iter().copied());
        let mut to_target = Vec::new();
        for occurrence in self.reporting.iter().copied() {
            if occurrence >= self.next_occurrence_index
                || !self.members.contains(&occurrence)
                || !seen.insert(occurrence)
            {
                continue;
            }
            self.active_dirty.insert(occurrence);
            if is_to_source(occurrence) {
                to_source.push(occurrence);
            } else {
                to_target.push(occurrence);
            }
        }
        to_source.extend(to_target);
        Some(to_source)
    }

    /// C++ clears `inDirtyList` immediately before updating the selected
    /// bind. Dirt for a later active occurrence therefore merges into its
    /// existing turn, while self/previous-occurrence dirt joins next pass.
    pub(crate) fn begin_occurrence(&mut self, occurrence_index: usize) {
        if self.active_dirty.remove(&occurrence_index) {
            self.pending.remove_data_bind(occurrence_index);
        }
    }

    pub(crate) fn finish_update(&mut self) {
        debug_assert!(self.processing);
        self.reporting.clear();
        self.active_dirty.clear();
        self.processing = false;
        // C++ flushes additions before removals. This also gives an
        // add-then-remove pair its chronological final state.
        let additions = std::mem::take(&mut self.pending_additions);
        for (occurrence_index, persisting) in additions {
            self.install_occurrence(occurrence_index, persisting);
        }
        let removals = std::mem::take(&mut self.pending_removals);
        for occurrence_index in removals {
            self.uninstall_occurrence(occurrence_index);
        }
    }

    /// End a Rust-only terminal-error path without losing the current
    /// occurrence or the untouched tail that C++ would have continued to
    /// visit after consuming an ordinary protected-call failure.
    pub(crate) fn abort_update(&mut self, remaining: impl IntoIterator<Item = usize>) {
        for occurrence in remaining {
            if occurrence < self.next_occurrence_index {
                self.pending.report_data_bind(occurrence);
            }
        }
        self.finish_update();
    }

    #[cfg(test)]
    fn report_for_test(&mut self, occurrence_index: usize) {
        self.next_occurrence_index = self.next_occurrence_index.max(occurrence_index + 1);
        self.members.insert(occurrence_index);
        self.pending.report_data_bind(occurrence_index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_partitions_to_source_and_preserves_partition_order() {
        let mut queue = RuntimeDataBindContainerQueue::default();
        for occurrence in [4, 1, 3, 2, 1, 0] {
            queue.report_for_test(occurrence);
        }

        let snapshot = queue
            .begin_update(|occurrence| occurrence % 2 == 1)
            .expect("outer update");
        assert_eq!(
            snapshot,
            vec![1, 3, 4, 2, 0],
            "to-source precedes to-target; first enqueue wins within each partition"
        );
        queue.finish_update();
    }

    #[test]
    fn persisting_occurrences_run_first_and_are_not_duplicated_by_dirt() {
        let mut queue = RuntimeDataBindContainerQueue::default();
        let mut persisting = RuntimeRetainedDataBind::new(1, false);
        assert_eq!(queue.add_data_bind(&mut persisting, true), 0);
        queue.report_for_test(2);
        queue.report_for_test(0);
        queue.report_for_test(1);

        assert_eq!(
            queue.begin_update(|occurrence| occurrence == 1),
            Some(vec![0, 1, 2])
        );
        queue.finish_update();
    }

    #[test]
    fn notifications_during_processing_wait_for_the_next_snapshot() {
        let mut queue = RuntimeDataBindContainerQueue::default();
        queue.report_for_test(0);

        assert_eq!(queue.begin_update(|_| false), Some(vec![0]));
        queue.report_for_test(1);
        assert_eq!(
            queue.begin_update(|_| true),
            None,
            "recursive updateDataBinds is rejected"
        );
        queue.finish_update();

        assert_eq!(queue.begin_update(|_| true), Some(vec![1]));
        queue.finish_update();
    }

    #[test]
    fn dirt_for_a_later_active_turn_is_consumed_without_a_second_pass() {
        let mut queue = RuntimeDataBindContainerQueue::default();
        queue.report_for_test(0);
        queue.report_for_test(1);
        assert_eq!(queue.begin_update(|_| false), Some(vec![0, 1]));

        queue.begin_occurrence(0);
        queue.report_for_test(1);
        // The report arrived before occurrence 1 was dequeued, so it merges
        // into occurrence 1's active C++ dirty-list turn.
        queue.begin_occurrence(1);
        queue.finish_update();

        assert_eq!(queue.begin_update(|_| false), Some(Vec::new()));
        queue.finish_update();
    }

    #[test]
    fn self_dirt_after_dequeue_waits_for_the_next_pass() {
        let mut queue = RuntimeDataBindContainerQueue::default();
        queue.report_for_test(0);
        assert_eq!(queue.begin_update(|_| false), Some(vec![0]));

        queue.begin_occurrence(0);
        queue.report_for_test(0);
        queue.finish_update();

        assert_eq!(queue.begin_update(|_| false), Some(vec![0]));
        queue.finish_update();
    }

    #[test]
    fn script_error_path_finishes_the_outer_pass_before_the_next_snapshot() {
        let mut queue = RuntimeDataBindContainerQueue::default();
        queue.report_for_test(0);
        queue.report_for_test(1);
        let selected = queue.begin_update(|_| false).expect("outer update");
        let result = (|| -> Result<(), ()> {
            assert_eq!(selected, vec![0, 1]);
            Err(()) // scripted converter/input protected-call failure
        })();
        queue.abort_update(selected);
        assert!(result.is_err());

        assert_eq!(queue.begin_update(|_| false), Some(vec![0, 1]));
        queue.finish_update();
    }

    // Literal behavioral ports of the dynamic-membership cases in pinned
    // `tests/unit_tests/runtime/data_bind_container_test.cpp`.

    #[test]
    fn add_registers_membership_and_persisting_state() {
        let mut queue = RuntimeDataBindContainerQueue::default();
        let mut to_target_bind = RuntimeRetainedDataBind::new(0, false);
        let mut to_source_bind = RuntimeRetainedDataBind::new(1, false);
        let to_target = queue.add_data_bind(&mut to_target_bind, false);
        let to_source = queue.add_data_bind(&mut to_source_bind, true);

        assert!(queue.contains(to_target));
        assert!(queue.contains(to_source));
        assert!(!queue.is_persisting(to_target));
        assert!(queue.is_persisting(to_source));
    }

    #[test]
    fn remove_clears_membership_persisting_and_dirty_state() {
        let mut queue = RuntimeDataBindContainerQueue::default();
        let mut data_bind = RuntimeRetainedDataBind::new(1, false);
        let occurrence = queue.add_data_bind(&mut data_bind, true);
        data_bind.mark_source_changed();

        queue.remove_data_bind(occurrence);

        assert!(!queue.contains(occurrence));
        assert!(!queue.is_persisting(occurrence));
        assert_eq!(queue.begin_update(|_| true), Some(Vec::new()));
        queue.finish_update();
    }

    #[test]
    fn add_during_update_is_deferred_and_flushed_after() {
        let mut queue = RuntimeDataBindContainerQueue::default();
        let mut driver_bind = RuntimeRetainedDataBind::new(1, false);
        let driver = queue.add_data_bind(&mut driver_bind, true);
        assert_eq!(queue.begin_update(|_| true), Some(vec![driver]));

        let mut added_bind = RuntimeRetainedDataBind::new(1, false);
        let added = queue.add_data_bind(&mut added_bind, true);
        assert!(!queue.contains(added));
        assert!(!queue.is_persisting(added));

        queue.finish_update();
        assert!(queue.contains(added));
        assert!(queue.is_persisting(added));
    }

    #[test]
    fn remove_during_update_is_deferred_and_flushed_after() {
        let mut queue = RuntimeDataBindContainerQueue::default();
        let mut driver_bind = RuntimeRetainedDataBind::new(1, false);
        let mut removed_bind = RuntimeRetainedDataBind::new(1, false);
        let driver = queue.add_data_bind(&mut driver_bind, true);
        let removed = queue.add_data_bind(&mut removed_bind, true);
        assert_eq!(queue.begin_update(|_| true), Some(vec![driver, removed]));

        queue.remove_data_bind(removed);
        assert!(queue.contains(removed));
        assert!(queue.is_persisting(removed));

        queue.finish_update();
        assert!(!queue.contains(removed));
        assert!(!queue.is_persisting(removed));
    }

    #[test]
    fn same_tick_add_then_remove_resolves_to_removed() {
        let mut queue = RuntimeDataBindContainerQueue::default();
        let mut driver_bind = RuntimeRetainedDataBind::new(1, false);
        let driver = queue.add_data_bind(&mut driver_bind, true);
        assert_eq!(queue.begin_update(|_| true), Some(vec![driver]));

        let mut transient_bind = RuntimeRetainedDataBind::new(1, false);
        let transient = queue.add_data_bind(&mut transient_bind, true);
        queue.remove_data_bind(transient);
        queue.finish_update();

        assert!(!queue.contains(transient));
        assert!(!queue.is_persisting(transient));
    }

    #[test]
    fn dirty_added_during_update_runs_on_next_tick() {
        let mut queue = RuntimeDataBindContainerQueue::default();
        let mut driver_bind = RuntimeRetainedDataBind::new(1, false);
        let mut dependent_bind = RuntimeRetainedDataBind::new(0, false);
        let driver = queue.add_data_bind(&mut driver_bind, true);
        let dependent = queue.add_data_bind(&mut dependent_bind, false);
        assert_eq!(queue.begin_update(|_| true), Some(vec![driver]));

        dependent_bind.mark_source_changed();
        queue.finish_update();

        assert_eq!(
            queue.begin_update(|occurrence| occurrence == driver),
            Some(vec![driver, dependent])
        );
        queue.begin_occurrence(dependent);
        queue.finish_update();
        assert_eq!(queue.begin_update(|_| false), Some(vec![driver]));
        queue.finish_update();
    }
}
