//! Active hosted-scene queries for Nuxie per-row collapse.
//! Visit counters compile out of shipping builds.

#[derive(Clone, Copy)]
#[doc(hidden)]
pub enum Walk {
    VideoEnumeration,
    VideoLookup,
    SemanticValidation,
    OwnerCapture,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[doc(hidden)]
pub struct WalkCounts {
    pub video_enumeration: u64,
    pub video_lookup: u64,
    pub semantic_validation: u64,
    pub owner_capture: u64,
}

#[cfg(any(test, feature = "test-support"))]
thread_local! {
    static COUNTS: std::cell::Cell<Option<WalkCounts>> = const { std::cell::Cell::new(None) };
}

/// Start or finish measuring hosted artboard visits on this thread.
#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
pub fn measure_walks(enabled: bool) -> WalkCounts {
    COUNTS.with(|counts| {
        counts
            .replace(enabled.then(WalkCounts::default))
            .unwrap_or_default()
    })
}

#[doc(hidden)]
#[inline]
pub fn record_walk(walk: Walk) {
    #[cfg(any(test, feature = "test-support"))]
    COUNTS.with(|counts| {
        if let Some(mut value) = counts.get() {
            match walk {
                Walk::VideoEnumeration => value.video_enumeration += 1,
                Walk::VideoLookup => value.video_lookup += 1,
                Walk::SemanticValidation => value.semantic_validation += 1,
                Walk::OwnerCapture => value.owner_capture += 1,
            }
            counts.set(Some(value));
        }
    });
    #[cfg(not(any(test, feature = "test-support")))]
    let _ = walk;
}

use crate::source::{
    artboard::RuntimeArtboardInstanceHandle, artboard_component_list::ArtboardComponentList,
    core::CoreObject,
};

thread_local! {
    static MEMBERSHIP_REVISION: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Structural edits invalidate occurrence maps even before the next advance.
pub(crate) fn membership_changed() {
    MEMBERSHIP_REVISION.with(|revision| revision.set(revision.get().wrapping_add(1)));
}

#[doc(hidden)]
pub fn membership_revision() -> u64 {
    MEMBERSHIP_REVISION.with(std::cell::Cell::get)
}

/// Keep upstream host enumeration intact. Only the Nuxie per-row collapse
/// operation removes retained rows from these per-frame query walks.
#[doc(hidden)]
pub fn active_instances(object: &dyn CoreObject) -> Vec<RuntimeArtboardInstanceHandle> {
    let Some(host) = object.as_artboard_host() else {
        return Vec::new();
    };
    if let Some(list) = object
        .as_registry_any()
        .downcast_ref::<ArtboardComponentList>()
    {
        return list.live_move_query_instances();
    }
    (0..host.artboard_count())
        .filter_map(|index| host.artboard_instance(index as i32))
        .collect()
}

impl ArtboardComponentList {
    fn live_move_query_instances(&self) -> Vec<RuntimeArtboardInstanceHandle> {
        if self.live_move_collapsed_mask.is_empty() {
            return (0..self.artboard_count())
                .filter_map(|index| self.artboard_instance(index as i32))
                .collect();
        }
        let mut instances = Vec::new();
        // Skip collapsed rows by bitmap word, without entering their artboards.
        for (word, collapsed) in self.live_move_collapsed_mask.iter().enumerate() {
            let mut active = !collapsed;
            while active != 0 {
                let index = word * 64 + active.trailing_zeros() as usize;
                if index >= self.artboard_count() {
                    break;
                }
                if let Some(instance) = self.artboard_instance(index as i32) {
                    instances.push(instance);
                }
                active &= active - 1;
            }
        }
        instances
    }
}

/// A row may already be collapsed when semantics are first enabled. Build its
/// retained nodes normally, then detach them before any snapshot can publish.
pub(crate) fn collapse_new_row_semantics(
    host: &crate::CoreHandle,
    index: i32,
    instance: &RuntimeArtboardInstanceHandle,
) {
    if host
        .with_downcast::<ArtboardComponentList, _>(|list| list.live_move_row_collapsed(index))
        .unwrap_or(false)
    {
        instance.collapse_semantic_boundary(true);
    }
}

/// Upstream's boundary wake restores only the artboard's own SemanticData.
/// Nuxie row collapse also detached descendant artboards. Restore those through
/// the active host graph, leaving independently collapsed descendants asleep.
pub(crate) fn resume_semantics(root: &RuntimeArtboardInstanceHandle) {
    let mut pending = vec![root.clone()];
    let mut seen = std::collections::HashSet::new();
    while let Some(board) = pending.pop() {
        if !seen.insert(board.core_handle()) {
            continue;
        }
        board.collapse_semantic_boundary(false);
        let hosts = board.with_artboard(|b| {
            b.nested_artboards()
                .into_iter()
                .chain(b.artboard_component_lists())
                .collect::<Vec<_>>()
        });
        for host in hosts {
            if let Some(children) = host.with(|object| {
                if object.component_is_collapsed() {
                    Vec::new()
                } else {
                    active_instances(object)
                }
            }) {
                pending.extend(children);
            }
        }
    }
}

/// Whether every enclosing Nuxie list row still participates in the scene.
pub(crate) fn is_active_artboard(board: &crate::CoreHandle) -> bool {
    let mut current = Some(board.clone());
    let mut seen = std::collections::HashSet::new();
    while let Some(board) = current {
        if !seen.insert(board.clone()) {
            return false;
        }
        let Some((host, parent)) =
            board.with_downcast::<crate::Artboard, _>(|b| (b.host(), b.parent_artboard()))
        else {
            return false;
        };
        if let Some(host) = host {
            let active = host.with_downcast::<ArtboardComponentList, _>(|list| {
                list.live_move_collapsed_rows.is_empty()
                    || list
                        .live_move_query_instances()
                        .iter()
                        .any(|row| row.core_handle() == board)
            });
            if active == Some(false) {
                return false;
            }
        }
        current = parent;
    }
    true
}
