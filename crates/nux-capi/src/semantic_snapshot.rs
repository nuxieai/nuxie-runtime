//! Owned, bounded captures of the runtime's occurrence-local semantic tree.

use super::*;
use nuxie::runtime::semantic::{
    semantic_data::SemanticData,
    semantic_manager::{RuntimeSemanticManagerHandle, SemanticManager},
    semantic_snapshot::SemanticsDiffNode,
    semantic_state::SemanticState,
};

const MAX_NODES: usize = 16_384;
const MAX_TEXT_BYTES: usize = 4 * 1024 * 1024;

/// Immutable capture. Text views remain valid until this handle is freed.
/// Like other runtime handles, all access is restricted to the creator thread.
pub struct NuxSemanticSnapshot {
    occurrence: std::rc::Weak<ArtboardOccurrence>,
    render_revision: u64,
    tree_version: u64,
    nodes: Vec<SemanticsDiffNode>,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NuxSemanticSnapshotInfo {
    pub struct_size: u32,
    pub render_revision: u64,
    pub tree_version: u64,
    pub node_count: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NuxSemanticNodeView {
    pub struct_size: u32,
    pub id: u32,
    pub parent_id: i32,
    pub sibling_index: u32,
    pub role: u32,
    pub state_flags: u32,
    pub trait_flags: u32,
    pub heading_level: u32,
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub label: NuxStringView,
    pub value: NuxStringView,
    pub hint: NuxStringView,
}

pub const NUX_SEMANTIC_SNAPSHOT_INFO_MIN_SIZE: usize =
    std::mem::offset_of!(NuxSemanticSnapshotInfo, node_count) + std::mem::size_of::<usize>();
pub const NUX_SEMANTIC_NODE_VIEW_MIN_SIZE: usize =
    std::mem::offset_of!(NuxSemanticNodeView, hint) + std::mem::size_of::<NuxStringView>();

fn copy_nodes(nodes: &[SemanticsDiffNode]) -> Result<Vec<SemanticsDiffNode>, NuxStatus> {
    if nodes.len() > MAX_NODES {
        return Err(NuxStatus::LimitExceeded);
    }
    let mut bytes = 0usize;
    for node in nodes {
        for len in [node.label.len(), node.value.len(), node.hint.len()] {
            bytes = bytes.checked_add(len).ok_or(NuxStatus::LimitExceeded)?;
            if bytes > MAX_TEXT_BYTES {
                return Err(NuxStatus::LimitExceeded);
            }
        }
    }
    Ok(nodes
        .iter()
        .map(|node| {
            let mut owned = node.clone();
            if node.state_flags & SemanticState::OBSCURED.0 != 0 {
                owned.value.clear();
            }
            owned
        })
        .collect())
}

/// Enable semantic tracking for the player's shared artboard occurrence.
/// Call before advancing/presenting. Static, animation, and state-machine
/// players sharing this occurrence observe the same tree.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_enable_semantics(player: *mut NuxPlayer) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = enter_status_handle!(player, HandleKind::Player);
        let player = unsafe { &*player };
        let _occurrence = match enter_occurrence(&player.artboard) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let artboard = player.artboard.instance.borrow().native_handle();
        if artboard.with_artboard(|artboard| artboard.semantic_manager().is_none()) {
            artboard.build_semantic_tree(
                Some(RuntimeSemanticManagerHandle::new(SemanticManager::new())),
                None,
            );
        }
        NuxStatus::Ok
    })
}

/// Capture only the current acknowledged presentation. Returns HANDLE_MISMATCH
/// for an unpresented revision and NOT_FOUND until semantics are enabled.
/// At most 16,384 nodes and 4 MiB of source UTF-8 text are accepted; larger
/// trees fail as a whole with LIMIT_EXCEEDED. Obscured values are omitted.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_semantic_snapshot(
    player: *const NuxPlayer,
    out_snapshot: *mut *mut NuxSemanticSnapshot,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_snapshot.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_snapshot = ptr::null_mut() };
        let _call = enter_status_handle!(player, HandleKind::Player);
        let player = unsafe { &*player };
        let _occurrence = match enter_occurrence(&player.artboard) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        if let Err(status) = player
            .artboard
            .refresh_renderer_domain_invalidation()
            .and_then(|()| player.artboard.refresh_bound_view_model_invalidation())
        {
            player.artboard.poisoned.set(true);
            return status;
        }
        let revision = player.artboard.render_revision.get();
        if revision == 0 || revision != player.artboard.presented_render_revision.get() {
            return NuxStatus::HandleMismatch;
        }
        let artboard = player.artboard.instance.borrow().native_handle();
        let Some(manager) = artboard.with_artboard(|artboard| artboard.semantic_manager()) else {
            return NuxStatus::NotFound;
        };
        let captured = manager.with_semantic_manager_mut(|manager| {
            let nodes = copy_nodes(manager.snapshot())?;
            Ok::<_, NuxStatus>((nodes, manager.version()))
        });
        let (nodes, tree_version) = match captured {
            Ok(captured) => captured,
            Err(status) => return status,
        };
        let snapshot = Box::into_raw(Box::new(NuxSemanticSnapshot {
            occurrence: Rc::downgrade(&player.artboard),
            render_revision: revision,
            tree_version,
            nodes,
        }));
        register_handle(snapshot, HandleKind::SemanticSnapshot, player.owner_thread);
        unsafe { *out_snapshot = snapshot };
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_semantic_snapshot_free(
    snapshot: *mut NuxSemanticSnapshot,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if snapshot.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(snapshot, HandleKind::SemanticSnapshot) {
            return status;
        }
        unsafe { drop(Box::from_raw(snapshot)) };
        NuxStatus::Ok
    })
}

/// Check that this capture still names this player's current presented
/// occurrence. A readable snapshot may outlive its player, but cannot then
/// authorize interaction or publication on a replacement occurrence.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_validate_semantic_snapshot(
    player: *const NuxPlayer,
    snapshot: *const NuxSemanticSnapshot,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _player = enter_status_handle!(player, HandleKind::Player);
        let _snapshot = enter_status_handle!(snapshot, HandleKind::SemanticSnapshot);
        let player = unsafe { &*player };
        let snapshot = unsafe { &*snapshot };
        let _occurrence = match enter_occurrence(&player.artboard) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        if let Err(status) = player
            .artboard
            .refresh_renderer_domain_invalidation()
            .and_then(|()| player.artboard.refresh_bound_view_model_invalidation())
        {
            player.artboard.poisoned.set(true);
            return status;
        }
        if !snapshot
            .occurrence
            .upgrade()
            .is_some_and(|occurrence| Rc::ptr_eq(&occurrence, &player.artboard))
            || snapshot.render_revision != player.artboard.render_revision.get()
            || snapshot.render_revision != player.artboard.presented_render_revision.get()
        {
            return NuxStatus::HandleMismatch;
        }
        NuxStatus::Ok
    })
}

/// Queue an authored semantic listener action: 0 tap, 1 increase, 2 decrease.
/// The host must subsequently step its occurrence's players and handle their
/// normal output journals. This does not execute a host command directly.
/// Stale captures return HANDLE_MISMATCH; absent or ineligible actions return
/// NOT_FOUND. Successful enqueue invalidates the capture for further actions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_queue_semantic_action(
    player: *mut NuxPlayer,
    snapshot: *const NuxSemanticSnapshot,
    node_id: u32,
    action: u32,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if action > 2 {
            return NuxStatus::InvalidArgument;
        }
        let validity = unsafe { nux_player_validate_semantic_snapshot(player, snapshot) };
        if validity != NuxStatus::Ok {
            return validity;
        }
        let _player = enter_status_handle!(player, HandleKind::Player);
        let _snapshot = enter_status_handle!(snapshot, HandleKind::SemanticSnapshot);
        let player = unsafe { &*player };
        let snapshot = unsafe { &*snapshot };
        let _occurrence = match enter_occurrence(&player.artboard) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        if !snapshot.nodes.iter().any(|node| node.id == node_id) {
            return NuxStatus::NotFound;
        }
        let artboard = player.artboard.instance.borrow().native_handle();
        let Some(manager) = artboard.with_artboard(|artboard| artboard.semantic_manager()) else {
            return NuxStatus::NotFound;
        };
        let node = manager.with_semantic_manager_mut(|manager| {
            manager.snapshot();
            if manager.version() != snapshot.tree_version {
                return Err(NuxStatus::HandleMismatch);
            }
            manager.node_by_id(node_id).ok_or(NuxStatus::NotFound)
        });
        let node = match node {
            Ok(node) => node,
            Err(status) => return status,
        };
        let mut ancestor = Some(node.clone());
        let mut remaining = MAX_NODES;
        while let Some(current) = ancestor {
            if remaining == 0 {
                return NuxStatus::LimitExceeded;
            }
            remaining -= 1;
            let current = current.borrow();
            if current.state_flags & (SemanticState::DISABLED.0 | SemanticState::HIDDEN.0) != 0 {
                return NuxStatus::NotFound;
            }
            ancestor = current.parent();
        }
        let Some(data) = node.borrow().semantic_data.clone() else {
            return NuxStatus::NotFound;
        };
        data.with_downcast::<SemanticData, _>(|data| {
            if !data.supports_semantic_action(action as u8) {
                return NuxStatus::NotFound;
            }
            if let Err(status) = player.artboard.invalidate_render() {
                return status;
            }
            match action {
                0 => data.fire_semantic_tap(),
                1 => data.fire_semantic_increase(),
                2 => data.fire_semantic_decrease(),
                _ => unreachable!(),
            }
            NuxStatus::Ok
        })
        .unwrap_or(NuxStatus::NotFound)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_semantic_snapshot_info(
    snapshot: *const NuxSemanticSnapshot,
    out_info: *mut NuxSemanticSnapshotInfo,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = enter_status_handle!(snapshot, HandleKind::SemanticSnapshot);
        let snapshot = unsafe { &*snapshot };
        let value = NuxSemanticSnapshotInfo {
            struct_size: std::mem::size_of::<NuxSemanticSnapshotInfo>() as u32,
            render_revision: snapshot.render_revision,
            tree_version: snapshot.tree_version,
            node_count: snapshot.nodes.len(),
        };
        unsafe { write_caller_struct(out_info, &value, NUX_SEMANTIC_SNAPSHOT_INFO_MIN_SIZE) }
            .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_semantic_snapshot_node(
    snapshot: *const NuxSemanticSnapshot,
    index: usize,
    out_node: *mut NuxSemanticNodeView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = enter_status_handle!(snapshot, HandleKind::SemanticSnapshot);
        let snapshot = unsafe { &*snapshot };
        let Some(node) = snapshot.nodes.get(index) else {
            return NuxStatus::NotFound;
        };
        let string = |value: &str| NuxStringView {
            data: value.as_ptr().cast(),
            len: value.len(),
        };
        let value = NuxSemanticNodeView {
            struct_size: std::mem::size_of::<NuxSemanticNodeView>() as u32,
            id: node.id,
            parent_id: node.parent_id,
            sibling_index: node.sibling_index,
            role: node.role,
            state_flags: node.state_flags,
            trait_flags: node.trait_flags,
            heading_level: node.heading_level,
            min_x: node.min_x,
            min_y: node.min_y,
            max_x: node.max_x,
            max_y: node.max_y,
            label: string(&node.label),
            value: string(&node.value),
            hint: string(&node.hint),
        };
        unsafe { write_caller_struct(out_node, &value, NUX_SEMANTIC_NODE_VIEW_MIN_SIZE) }
            .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_copy_owns_text_and_omits_obscured_values() {
        let mut source = vec![SemanticsDiffNode {
            label: "Password".into(),
            value: "private".into(),
            state_flags: SemanticState::OBSCURED.0,
            ..Default::default()
        }];
        let capture = copy_nodes(&source).unwrap();
        source[0].label.clear();
        assert_eq!(capture[0].label, "Password");
        assert_eq!(capture[0].value, "");
        assert_eq!(source[0].value, "private");
        assert_eq!(
            copy_nodes(&vec![SemanticsDiffNode::default(); MAX_NODES + 1]).unwrap_err(),
            NuxStatus::LimitExceeded
        );
        source[0].hint = "x".repeat(MAX_TEXT_BYTES);
        assert_eq!(copy_nodes(&source).unwrap_err(), NuxStatus::LimitExceeded);
    }

    #[test]
    fn snapshot_handle_enforces_lifetime_and_creator_thread() {
        let snapshot = Box::into_raw(Box::new(NuxSemanticSnapshot {
            occurrence: std::rc::Weak::new(),
            render_revision: 7,
            tree_version: 2,
            nodes: vec![SemanticsDiffNode {
                label: "Continue".into(),
                ..Default::default()
            }],
        }));
        register_handle(
            snapshot,
            HandleKind::SemanticSnapshot,
            std::thread::current().id(),
        );
        let address = snapshot as usize;
        let wrong_thread = std::thread::spawn(move || unsafe {
            nux_semantic_snapshot_free(address as *mut NuxSemanticSnapshot)
        })
        .join()
        .unwrap();
        assert_ne!(wrong_thread, NuxStatus::Ok);
        let mut view = NuxSemanticNodeView {
            struct_size: std::mem::size_of::<NuxSemanticNodeView>() as u32,
            ..Default::default()
        };
        unsafe {
            assert_eq!(
                nux_semantic_snapshot_node(snapshot, 0, &mut view),
                NuxStatus::Ok
            );
            assert_eq!(
                std::slice::from_raw_parts(view.label.data.cast::<u8>(), view.label.len),
                b"Continue"
            );
            assert_eq!(
                nux_semantic_snapshot_node(snapshot, 1, &mut view),
                NuxStatus::NotFound
            );
            assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
            assert_ne!(
                nux_semantic_snapshot_node(snapshot, 0, &mut view),
                NuxStatus::Ok
            );
            assert_eq!(nux_semantic_snapshot_free(ptr::null_mut()), NuxStatus::Ok);
        }
    }
}
