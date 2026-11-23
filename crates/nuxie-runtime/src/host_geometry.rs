//! Owned host observations derived from translated runtime occurrences.

/// One exact runtime-local step in a geometry hit path.
///
/// Local ids are scoped to an artboard definition. The containing artboard's
/// file-global id keeps paths through nested and repeated occurrences
/// unambiguous without introducing a second graph owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeGeometryHitPathSegment {
    pub artboard_global_id: u32,
    pub local_id: usize,
}

/// One repeated component-list item on the descent to a concrete hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeGeometryHitOccurrence {
    pub artboard_global_id: u32,
    pub host_local_id: usize,
    pub item_index: usize,
    /// Stable identity of the translated runtime occurrence at observation
    /// time. `item_index` alone is not durable across list topology changes.
    pub occurrence_identity: u64,
}

/// One keyed list row on the descent to an occurrence: the list's host
/// object, by its local id in the artboard with `artboard_global_id`, and the
/// key the row was mounted by.
///
/// A key is bytes the engine compares only for equality. A path through a
/// keyed list carries one of these for each keyed list step, in path order,
/// beside the path's segments: a report from a keyed row carries them that
/// way (`StateMachineEventContext::rows`), and a driver hands them back in a
/// `SceneObject`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeListRowKey {
    pub artboard_global_id: u32,
    pub host_local_id: usize,
    pub key: Box<[u8]>,
}
