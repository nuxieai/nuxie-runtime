//! Immutable-membership projection of pinned C++ `DataBindContainer`.
//!
//! FL-C4 constructs every ordinary state-machine and cloned scripted-object
//! bind before the first update and retains that set until owner destruction.
//! For that fixed set, an update swaps the current queue into a reporting
//! buffer, processes all to-source occurrences before all to-target
//! occurrences while preserving enqueue order inside each partition, and
//! leaves dirt raised during traversal pending for the next call
//! (`src/data_bind/data_bind_container.cpp:97-107,115-203,245-269`).
//! Dynamic membership remains on the pending whole owner in FL-D.

use std::collections::BTreeSet;

use crate::retained_data_bind::RuntimeRetainedDataBind;
use crate::view_model_cell::RuntimeCellNotificationQueue;

#[derive(Debug, Default)]
pub(crate) struct RuntimeDataBindContainerQueue {
    pending: RuntimeCellNotificationQueue,
    reporting: Vec<usize>,
    active_dirty: BTreeSet<usize>,
    persisting: Vec<usize>,
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
        // FL-C4 reaches only ScriptedListenerAction/ScriptedDataConverter
        // membership, which C++ finishes constructing before the first update
        // and retains until owner destruction (`scripted_object.cpp:558-586`;
        // `state_machine_instance.cpp:2072-2082,2141-2199`). Dynamic
        // add/remove during processing remains on the pending FL-D whole
        // DataBindContainer owner.
        assert!(
            !self.processing,
            "dynamic DataBindContainer membership belongs to FL-D"
        );
        let index = self.next_occurrence_index;
        self.next_occurrence_index += 1;
        data_bind.report_source_dirt_to(&self.pending, index);
        if persisting {
            self.persisting.push(index);
        }
        index
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
        let mut to_source = self.persisting.clone();
        seen.extend(to_source.iter().copied());
        let mut to_target = Vec::new();
        for occurrence in self.reporting.iter().copied() {
            if occurrence >= self.next_occurrence_index || !seen.insert(occurrence) {
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
}
