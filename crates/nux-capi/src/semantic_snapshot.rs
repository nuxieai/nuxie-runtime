//! Owned, bounded captures of the runtime's occurrence-local semantic tree.

use super::*;
use nuxie::runtime::semantic::{
    semantic_data::SemanticData,
    semantic_manager::{RuntimeSemanticManagerHandle, SemanticManager},
    semantic_node::{SemanticNode, SemanticNodeRef},
    semantic_snapshot::SemanticsDiffNode,
    semantic_state::SemanticState,
};

const MAX_NODES: usize = 16_384;
const MAX_TEXT_BYTES: usize = 4 * 1024 * 1024;

/// Property used by either supported editable endpoint, not by a display run.
fn editable_string_key(object: &nuxie::runtime::core::CoreHandle) -> Option<i32> {
    use nuxie::runtime::{
        core::CoreType,
        custom_property_string::CustomPropertyString,
        generated::{
            custom_property_string_base::CustomPropertyStringBase,
            text::text_input_base::TextInputBase,
        },
        text::text_input::TextInput,
    };
    if object.is_type_of(TextInput::TYPE_KEY) {
        Some(i32::from(TextInputBase::TEXT_PROPERTY_KEY))
    } else if object.is_type_of(CustomPropertyString::TYPE_KEY) {
        Some(i32::from(
            CustomPropertyStringBase::PROPERTY_VALUE_PROPERTY_KEY,
        ))
    } else {
        None
    }
}

fn is_within_field(
    input: &nuxie::runtime::core::CoreHandle,
    field: &nuxie::runtime::core::CoreHandle,
) -> bool {
    let mut current = Some(input.clone());
    let mut visited = std::collections::HashSet::new();
    while let Some(object) = current {
        if object.identity_key() == field.identity_key() {
            return true;
        }
        if !visited.insert(object.identity_key()) || visited.len() > MAX_NODES {
            return false;
        }
        current = object
            .with(|object| {
                object
                    .as_component()
                    .and_then(|component| component.parent_handle())
            })
            .flatten();
    }
    false
}

/// Resolve an editable value in the field's own occurrence.
/// The caller must first validate its presented snapshot and field node id.
fn field_string_property(
    node: &SemanticNodeRef,
    name: &str,
) -> Result<nuxie::runtime::core::CoreHandle, NuxStatus> {
    use nuxie::runtime::{artboard::Artboard, core::CoreType, text::text_input::TextInput};
    if name.is_empty() {
        return Err(NuxStatus::InvalidArgument);
    }
    if name.len() > 4096 {
        return Err(NuxStatus::LimitExceeded);
    }
    let data = eligible_data(node)?;
    if !data
        .with_downcast::<SemanticData, _>(|data| data.base.role() == NUX_SEMANTIC_ROLE_TEXT_FIELD)
        .unwrap_or(false)
    {
        return Err(NuxStatus::NotFound);
    }
    let owner = node
        .borrow()
        .core_owner
        .clone()
        .ok_or(NuxStatus::NotFound)?;
    let artboard = owner
        .with(|owner| {
            owner
                .as_component()
                .and_then(|component| component.artboard_handle())
        })
        .flatten()
        .ok_or(NuxStatus::NotFound)?;
    let matches = artboard
        .with_downcast::<Artboard, _>(|artboard| {
            artboard
                .objects()
                .iter()
                .flatten()
                .filter(|object| {
                    editable_string_key(object).is_some()
                        && (!object.is_type_of(TextInput::TYPE_KEY)
                            || is_within_field(object, &owner))
                        && object
                            .with(|object| {
                                object
                                    .as_component()
                                    .is_some_and(|component| component.name() == name)
                            })
                            .unwrap_or(false)
                })
                .take(2)
                .cloned()
                .collect::<Vec<_>>()
        })
        .ok_or(NuxStatus::NotFound)?;
    match matches.as_slice() {
        [property] => Ok(property.clone()),
        [] => Err(NuxStatus::NotFound),
        _ => Err(NuxStatus::InvalidArgument),
    }
}

fn eligible_data(node: &SemanticNodeRef) -> Result<nuxie::runtime::core::CoreHandle, NuxStatus> {
    if !SemanticNode::is_action_eligible(node) {
        return Err(NuxStatus::NotFound);
    }
    node.borrow()
        .semantic_data
        .clone()
        .ok_or(NuxStatus::NotFound)
}

pub(super) unsafe fn with_presented_field_property(
    player: *const NuxPlayer,
    snapshot: *const NuxSemanticSnapshot,
    node_id: u32,
    name: NuxStringView,
    writable: bool,
    use_property: impl FnOnce(&ArtboardOccurrence, &nuxie::runtime::core::CoreHandle) -> NuxStatus,
) -> NuxStatus {
    if name.len > 4096 {
        return NuxStatus::LimitExceeded;
    }
    let name = match with_utf8_view(name, str::to_owned) {
        Ok(name) => name,
        Err(status) => return status,
    };
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
    if !snapshot
        .nodes
        .iter()
        .any(|node| node.id == node_id && node.role == NUX_SEMANTIC_ROLE_TEXT_FIELD)
    {
        return NuxStatus::NotFound;
    }
    let artboard = player.artboard.instance.borrow().native_handle();
    let Some(manager) = artboard.with_artboard(|artboard| artboard.semantic_manager()) else {
        return NuxStatus::NotFound;
    };
    let node = manager.with_semantic_manager(|manager| {
        if manager.version() != snapshot.tree_version {
            return Err(NuxStatus::HandleMismatch);
        }
        manager.node_by_id(node_id).ok_or(NuxStatus::NotFound)
    });
    let property = match node.and_then(|node| {
        if writable && node.borrow().state_flags & SemanticState::READ_ONLY.0 != 0 {
            return Err(NuxStatus::NotFound);
        }
        field_string_property(&node, &name)
    }) {
        Ok(property) => property,
        Err(status) => return status,
    };
    use_property(&player.artboard, &property)
}

/// Acquire the presented field's existing ViewModel, without cloning its state.
/// Missing ownership fails rather than using root. This read does not invalidate
/// the capture. The caller must free the handle with nux_view_model_instance_free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_field_view_model_instance(
    player: *const NuxPlayer,
    snapshot: *const NuxSemanticSnapshot,
    node_id: u32,
    name: NuxStringView,
    out_instance: *mut *mut NuxViewModelInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_instance.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_instance = ptr::null_mut() };
        unsafe {
            with_presented_field_property(
                player,
                snapshot,
                node_id,
                name,
                false,
                |occurrence, property| {
                    use nuxie::runtime::{
                        artboard::Artboard, data_bind::data_context::DataContext,
                    };
                    let owner = property
                        .with(|property| {
                            property
                                .as_component()
                                .and_then(|component| component.artboard_handle())
                        })
                        .flatten()
                        .and_then(|artboard| {
                            artboard
                                .with_downcast::<Artboard, _>(|artboard| {
                                    artboard.data_context().and_then(|context| {
                                        context.with_context(DataContext::main_view_model_instance)
                                    })
                                })
                                .flatten()
                        });
                    let Some(owner) = owner else {
                        return NuxStatus::NotFound;
                    };
                    let file = occurrence.instance.borrow().native_file();
                    let Some(owner) =
                        RuntimeOwnedViewModelInstance::from_native(file.clone(), owner)
                    else {
                        return NuxStatus::NotFound;
                    };
                    let player = &*player;
                    let handle = Box::into_raw(Box::new(NuxViewModelInstance {
                        schema_index: owner.view_model_index(),
                        identity: owner.instance_identity(),
                        instance: RuntimeOwnedViewModelHandle::new(owner),
                        file,
                        view_model_catalog: Arc::clone(&player.view_model_catalog),
                        owner_thread: player.owner_thread,
                        file_provenance: Arc::clone(&player.file_provenance),
                        binding_provenance: Some(Arc::clone(&player.provenance)),
                        provenance: Arc::new(()),
                    }));
                    register_handle(handle, HandleKind::ViewModel, player.owner_thread);
                    *out_instance = handle;
                    NuxStatus::Ok
                },
            )
        }
    })
}

/// Copy a field's editable UTF-8 value into caller-owned memory, without
/// a terminator. A null buffer with zero capacity queries the required length.
/// Insufficient capacity returns LIMIT_EXCEEDED without copying partial text.
/// This explicit execution read is not included in semantic/diagnostic captures.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_field_string_copy(
    player: *const NuxPlayer,
    snapshot: *const NuxSemanticSnapshot,
    node_id: u32,
    name: NuxStringView,
    buffer: *mut u8,
    capacity: usize,
    out_length: *mut usize,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_length.is_null() || (buffer.is_null() && capacity != 0) {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_length = 0;
        }
        unsafe {
            with_presented_field_property(player, snapshot, node_id, name, false, |_, property| {
                use nuxie::runtime::generated::core_registry::CoreRegistry;
                let Some(key) = editable_string_key(property) else {
                    return NuxStatus::NotFound;
                };
                let Some(value) = CoreRegistry::get_string_handle(property, key) else {
                    return NuxStatus::NotFound;
                };
                if value.len() > 1024 * 1024 {
                    return NuxStatus::LimitExceeded;
                }
                *out_length = value.len();
                if buffer.is_null() && capacity == 0 {
                    return NuxStatus::Ok;
                }
                if capacity < value.len() {
                    return NuxStatus::LimitExceeded;
                }
                if !value.is_empty() {
                    ptr::copy_nonoverlapping(value.as_ptr(), buffer, value.len());
                }
                NuxStatus::Ok
            })
        }
    })
}

/// Edit a presented field's value through its native callback.
/// A changed value invalidates the capture; step/present before another edit.
/// Native bindings perform reverse conversion on their normal settlement path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_field_string_set(
    player: *mut NuxPlayer,
    snapshot: *const NuxSemanticSnapshot,
    node_id: u32,
    name: NuxStringView,
    value: NuxStringView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if value.len > 1024 * 1024 {
            return NuxStatus::LimitExceeded;
        }
        let value = match with_utf8_view(value, str::to_owned) {
            Ok(value) => value,
            Err(status) => return status,
        };
        unsafe {
            with_presented_field_property(
                player,
                snapshot,
                node_id,
                name,
                true,
                |occurrence, property| {
                    use nuxie::runtime::generated::core_registry::CoreRegistry;
                    let Some(key) = editable_string_key(property) else {
                        return NuxStatus::NotFound;
                    };
                    let Some(before) = CoreRegistry::get_string_handle(property, key) else {
                        return NuxStatus::NotFound;
                    };
                    if before == value {
                        return NuxStatus::Ok;
                    }
                    if let Err(status) = occurrence.invalidate_render() {
                        return status;
                    }
                    if CoreRegistry::set_string_handle(property, key, value) {
                        NuxStatus::Ok
                    } else {
                        NuxStatus::RuntimeError
                    }
                },
            )
        }
    })
}

/// Immutable capture. Text views remain valid until this handle is freed.
/// Like other runtime handles, all access is restricted to the creator thread.
pub struct NuxSemanticSnapshot {
    occurrence: std::rc::Weak<ArtboardOccurrence>,
    render_revision: u64,
    tree_version: u64,
    nodes: Vec<SemanticsDiffNode>,
    actions: Vec<u32>,
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
    /// Bit 0: tap; bit 1: increase; bit 2: decrease. Zero for ineligible nodes.
    pub actions: u32,
}

pub const NUX_SEMANTIC_SNAPSHOT_INFO_MIN_SIZE: usize =
    std::mem::offset_of!(NuxSemanticSnapshotInfo, node_count) + std::mem::size_of::<usize>();
pub const NUX_SEMANTIC_NODE_VIEW_MIN_SIZE: usize =
    std::mem::offset_of!(NuxSemanticNodeView, actions) + std::mem::size_of::<u32>();

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
        .map(|node| SemanticsDiffNode {
            id: node.id,
            role: node.role,
            parent_id: node.parent_id,
            sibling_index: node.sibling_index,
            state_flags: node.state_flags,
            trait_flags: node.trait_flags,
            heading_level: node.heading_level,
            min_x: node.min_x,
            min_y: node.min_y,
            max_x: node.max_x,
            max_y: node.max_y,
            label: node.label.clone(),
            hint: node.hint.clone(),
            value: if node.state_flags & SemanticState::OBSCURED.0 != 0 {
                String::new()
            } else {
                node.value.clone()
            },
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
        use nuxie::runtime::semantic::semantic_provider::{
            SemanticGeometryError, validate_semantic_geometry,
        };
        if let Err(error) = validate_semantic_geometry(&artboard.core_handle()) {
            return match error {
                SemanticGeometryError::LimitExceeded => NuxStatus::LimitExceeded,
                SemanticGeometryError::InvalidPath => NuxStatus::RuntimeError,
            };
        }
        let captured = manager.with_semantic_manager_mut(|manager| {
            let nodes = copy_nodes(manager.snapshot())?;
            Ok::<_, NuxStatus>((nodes, manager.version()))
        });
        let (nodes, tree_version) = match captured {
            Ok(captured) => captured,
            Err(status) => return status,
        };
        let actions = nodes
            .iter()
            .map(|node| {
                let Some(node) =
                    manager.with_semantic_manager(|manager| manager.node_by_id(node.id))
                else {
                    return 0;
                };
                let Ok(data) = eligible_data(&node) else {
                    return 0;
                };
                data.with_downcast::<SemanticData, _>(|data| {
                    (0..3)
                        .filter(|action| data.supports_semantic_action(*action))
                        .fold(0u32, |mask, action| mask | (1 << action))
                })
                .unwrap_or(0)
            })
            .collect();
        let snapshot = Box::into_raw(Box::new(NuxSemanticSnapshot {
            occurrence: Rc::downgrade(&player.artboard),
            render_revision: revision,
            tree_version,
            nodes,
            actions,
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

/// Associate an exact-name root text run with its presented semantic text-field node.
/// The lookup uses the same root scope as text mutation and the native text owner
/// or its enclosing field ancestors, never labels or geometry.
/// Missing/non-field/hidden nodes return NOT_FOUND; ambiguous names or owners return
/// INVALID_ARGUMENT. Stale captures return HANDLE_MISMATCH. No text value is read.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_semantic_node_for_text_run(
    player: *const NuxPlayer,
    snapshot: *const NuxSemanticSnapshot,
    name: NuxStringView,
    out_node_id: *mut u32,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_node_id.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_node_id = 0 };
        if name.len > 4096 {
            return NuxStatus::LimitExceeded;
        }
        let name = match with_utf8_view(name, str::to_owned) {
            Ok(name) if !name.is_empty() => name,
            Ok(_) => return NuxStatus::InvalidArgument,
            Err(status) => return status,
        };
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
        let owner = match data_binding::root_text_owner(&player.artboard, &name) {
            Ok(owner) => owner,
            Err(status) => return status,
        };
        let artboard = player.artboard.instance.borrow().native_handle();
        let mut owners = std::collections::HashSet::new();
        let mut ancestor = Some(owner);
        while let Some(owner) = ancestor {
            if !owners.insert(owner.identity_key()) {
                return NuxStatus::InvalidArgument;
            }
            ancestor = owner
                .with(|object| {
                    object
                        .as_component()
                        .and_then(|component| component.parent_handle())
                })
                .flatten();
        }
        let Some(manager) = artboard.with_artboard(|artboard| artboard.semantic_manager()) else {
            return NuxStatus::NotFound;
        };
        let matches = manager.with_semantic_manager(|manager| {
            if manager.version() != snapshot.tree_version {
                return Err(NuxStatus::HandleMismatch);
            }
            Ok(snapshot
                .nodes
                .iter()
                .filter_map(|captured| {
                    if captured.role != NUX_SEMANTIC_ROLE_TEXT_FIELD {
                        return None;
                    }
                    let node = manager.node_by_id(captured.id)?;
                    node.borrow()
                        .core_owner
                        .as_ref()
                        .is_some_and(|owner| owners.contains(&owner.identity_key()))
                        .then_some(captured.id)
                })
                .collect::<Vec<_>>())
        });
        match matches {
            Err(status) => status,
            Ok(ids) if ids.len() > 1 => NuxStatus::InvalidArgument,
            Ok(ids) => match ids.first() {
                Some(id) => {
                    unsafe { *out_node_id = *id };
                    NuxStatus::Ok
                }
                None => NuxStatus::NotFound,
            },
        }
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
        let data = match eligible_data(&node) {
            Ok(data) => data,
            Err(status) => return status,
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
            actions: snapshot.actions[index],
        };
        unsafe { write_caller_struct(out_node, &value, NUX_SEMANTIC_NODE_VIEW_MIN_SIZE) }
            .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    mod fixture {
        include!("../tests/support/semantic_text.rs");
    }

    #[test]
    fn field_string_edits_are_local_to_repeated_occurrences() {
        check_repeated_field_edits(fixture::repeated_nonvisual_fields(), None, false, 1);
    }

    #[test]
    fn native_input_edits_are_local_to_repeated_occurrences() {
        for obscured in [false, true] {
            check_repeated_field_edits(
                fixture::repeated_native_input_fields(obscured),
                Some(obscured),
                false,
                1,
            );
            check_repeated_field_edits(
                fixture::transformed_native_input_fields(obscured),
                Some(obscured),
                true,
                1,
            );
        }
    }

    #[test]
    fn native_inputs_with_same_name_resolve_within_their_field_owner() {
        for obscured in [false, true] {
            check_repeated_field_edits(
                fixture::repeated_pair_native_input_fields(obscured),
                Some(obscured),
                false,
                2,
            );
        }
    }

    fn check_repeated_field_edits(
        bytes: Vec<u8>,
        native_input: Option<bool>,
        transformed: bool,
        fields_per_instance: usize,
    ) {
        check_repeated_field_ownership(
            bytes,
            native_input,
            transformed,
            fields_per_instance,
            false,
        );
    }

    #[test]
    fn native_field_owner_is_local_to_each_repeated_artboard() {
        check_repeated_field_ownership(
            fixture::repeated_owned_native_input_fields(),
            Some(false),
            false,
            2,
            true,
        );
    }

    fn check_repeated_field_ownership(
        bytes: Vec<u8>,
        native_input: Option<bool>,
        transformed: bool,
        fields_per_instance: usize,
        owned: bool,
    ) {
        unsafe {
            let mut file = ptr::null_mut();
            assert_eq!(
                nux_file_import(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &mut file
                ),
                NuxStatus::Ok
            );
            let mut instance = ptr::null_mut();
            assert_eq!(
                nux_artboard_instance_new(file, 1, &mut instance),
                NuxStatus::Ok
            );
            let mut expected_owners = Vec::new();
            let mut expected_models = Vec::new();
            if owned {
                // Static players do not auto-bind authored defaults. Bind each
                // nested occurrence exactly as a host does before presentation.
                let instance = &*instance;
                let artboard = instance.occurrence.instance.borrow();
                let native = artboard.native_handle();
                let hosts = native.with_artboard(|artboard| artboard.base.nested_artboards());
                for host in hosts {
                    let child = host
                        .with(|object| object.as_artboard_host().unwrap().artboard_instance(0))
                        .flatten()
                        .unwrap();
                    let model =
                        RuntimeOwnedViewModelInstance::new(artboard.native_file(), 0).unwrap();
                    expected_owners.push(model.instance_identity());
                    child.bind_view_model_instance(Some(model.native_handle()));
                    expected_models.push(model);
                }
            }
            let mut player = ptr::null_mut();
            assert_eq!(nux_player_new_static(instance, &mut player), NuxStatus::Ok);
            assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
            let capture = || {
                let step = NuxPlayerStep {
                    struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
                    ..Default::default()
                };
                let mut result = ptr::null_mut();
                assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
                let mut info = NuxPlayerSchedulingInfo {
                    struct_size: std::mem::size_of::<NuxPlayerSchedulingInfo>() as u32,
                    ..Default::default()
                };
                assert_eq!(
                    nux_player_step_result_scheduling(result, &mut info),
                    NuxStatus::Ok
                );
                assert_eq!(
                    nux_player_acknowledge_presented(player, info.render_revision),
                    NuxStatus::Ok
                );
                assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
                let mut snapshot = ptr::null_mut();
                assert_eq!(
                    nux_player_semantic_snapshot(player, &mut snapshot),
                    NuxStatus::Ok
                );
                snapshot
            };
            let mut snapshot = capture();
            let mut fields = (&(*snapshot).nodes)
                .iter()
                .filter(|node| node.role == NUX_SEMANTIC_ROLE_TEXT_FIELD)
                .collect::<Vec<_>>();
            fields.sort_by(|a, b| {
                a.min_x
                    .total_cmp(&b.min_x)
                    .then(a.min_y.total_cmp(&b.min_y))
            });
            assert_eq!(
                fields.len(),
                2 * fields_per_instance,
                "all nested fields are presented"
            );
            let ids = fields.iter().map(|field| field.id).collect::<Vec<_>>();
            assert_ne!(ids[0], ids[1]);
            let view = |text: &str| NuxStringView {
                data: text.as_ptr().cast(),
                len: text.len(),
            };
            let mut owners = Vec::new();
            for (index, &id) in ids.iter().enumerate() {
                let mut owner_handle = ptr::null_mut();
                assert_eq!(
                    nux_player_field_view_model_instance(
                        player,
                        snapshot,
                        id,
                        view("editable"),
                        &mut owner_handle
                    ),
                    if owned {
                        NuxStatus::Ok
                    } else {
                        NuxStatus::NotFound
                    },
                    "only fields with an actual context have an owner"
                );
                let mut owner_id = 0;
                if owned {
                    assert_eq!(
                        nux_view_model_instance_identity(owner_handle, &mut owner_id),
                        NuxStatus::Ok
                    );
                    assert_ne!(owner_id, 0);
                    assert_eq!(nux_view_model_instance_free(owner_handle), NuxStatus::Ok);
                } else {
                    assert!(owner_handle.is_null());
                }
                owners.push(owner_id);
                let x = if index < fields_per_instance {
                    60.0
                } else {
                    240.0
                };
                let y = 30.0 + (index % fields_per_instance) as f32 * 50.0;
                let mut geometry = NuxTextInputGeometry {
                    struct_size: std::mem::size_of::<NuxTextInputGeometry>() as u32,
                    ..Default::default()
                };
                let status = nux_player_text_input_geometry(
                    player,
                    snapshot,
                    id,
                    view("editable"),
                    &mut geometry,
                );
                if let Some(obscured) = native_input {
                    assert_eq!(status, NuxStatus::Ok);
                    let expected = if transformed {
                        [0.0, 2.0, -3.0, 0.0, x - 33.0, 44.0]
                    } else {
                        [1.0, 0.0, 0.0, 1.0, x, y]
                    };
                    for (actual, expected) in geometry.world_transform.into_iter().zip(expected) {
                        assert!(
                            (actual - expected).abs() < 0.0001,
                            "actual {actual}, expected {expected}"
                        );
                    }
                    assert_eq!(geometry.obscured, u32::from(obscured));
                    assert_eq!(
                        geometry.has_first_baseline, 0,
                        "fontless fixture has no shaped baseline"
                    );
                } else {
                    assert_eq!(status, NuxStatus::NotFound);
                }
            }
            if owned {
                assert_eq!(owners[0], expected_owners[0]);
                assert_eq!(owners[2], expected_owners[1]);
                assert_eq!(
                    owners[0], owners[1],
                    "fields in one occurrence share their owner"
                );
                assert_eq!(owners[2], owners[3]);
                assert_ne!(
                    owners[0], owners[2],
                    "repeated occurrences have distinct owners"
                );
                let other_before = expected_models[1].string_value_by_property_name("answer");
                let mut handle = ptr::null_mut();
                assert_eq!(
                    nux_player_field_view_model_instance(
                        player,
                        snapshot,
                        ids[0],
                        view("editable"),
                        &mut handle
                    ),
                    NuxStatus::Ok
                );
                let value = b"changed in place";
                let mutation = NuxViewModelMutation {
                    kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
                    instance: handle,
                    path: view("answer"),
                    bytes_value: NuxByteView {
                        data: value.as_ptr(),
                        len: value.len(),
                    },
                    ..NuxViewModelMutation::default()
                };
                let batch = NuxViewModelMutationBatch {
                    mutations: &mutation,
                    mutation_count: 1,
                    ..NuxViewModelMutationBatch::default()
                };
                let mut result = ptr::null_mut();
                assert_eq!(nux_view_model_mutate(&batch, &mut result), NuxStatus::Ok);
                assert_eq!(nux_view_model_mutation_result_free(result), NuxStatus::Ok);
                assert_eq!(nux_view_model_instance_free(handle), NuxStatus::Ok);
                assert_eq!(
                    expected_models[0]
                        .string_value_by_property_name("answer")
                        .as_deref(),
                    Some(value.as_slice()),
                    "the acquired handle edits the existing occurrence, not a copy"
                );
                assert_eq!(
                    expected_models[1].string_value_by_property_name("answer"),
                    other_before
                );
                assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
                snapshot = capture();
            }
            for edit in ["first edit", "日本語 e\u{301}🙂", ""] {
                assert_eq!(
                    nux_player_field_string_set(
                        player,
                        snapshot,
                        ids[0],
                        view("editable"),
                        view(edit)
                    ),
                    NuxStatus::Ok
                );
                let mut stale_geometry = NuxTextInputGeometry {
                    struct_size: std::mem::size_of::<NuxTextInputGeometry>() as u32,
                    ..Default::default()
                };
                let mut stale_owner = ptr::null_mut();
                assert_eq!(
                    nux_player_field_view_model_instance(
                        player,
                        snapshot,
                        ids[0],
                        view("editable"),
                        &mut stale_owner
                    ),
                    NuxStatus::HandleMismatch
                );
                assert!(stale_owner.is_null());
                assert_eq!(
                    nux_player_text_input_geometry(
                        player,
                        snapshot,
                        ids[0],
                        view("editable"),
                        &mut stale_geometry
                    ),
                    NuxStatus::HandleMismatch
                );
                assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
                snapshot = capture();
                for (index, &id) in ids.iter().enumerate() {
                    if owned {
                        let mut handle = ptr::null_mut();
                        assert_eq!(
                            nux_player_field_view_model_instance(
                                player,
                                snapshot,
                                id,
                                view("editable"),
                                &mut handle
                            ),
                            NuxStatus::Ok
                        );
                        let mut owner = 0;
                        assert_eq!(
                            nux_view_model_instance_identity(handle, &mut owner),
                            NuxStatus::Ok
                        );
                        assert_eq!(nux_view_model_instance_free(handle), NuxStatus::Ok);
                        assert_eq!(
                            owner, owners[index],
                            "settling a value does not replace its owning instance"
                        );
                    }
                    let expected = if index == 0 { edit } else { "editable value" };
                    let mut bytes = [0u8; 64];
                    let mut length = 0;
                    assert_eq!(
                        nux_player_field_string_copy(
                            player,
                            snapshot,
                            id,
                            view("editable"),
                            bytes.as_mut_ptr(),
                            bytes.len(),
                            &mut length
                        ),
                        NuxStatus::Ok
                    );
                    assert_eq!(&bytes[..length], expected.as_bytes());
                }
                assert!(
                    (&(*snapshot).nodes)
                        .iter()
                        .all(|node| node.value.is_empty()),
                    "editable text is not included in semantic captures"
                );
            }
            assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
            assert_eq!(nux_player_free(player), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_free(instance), NuxStatus::Ok);
            assert_eq!(nux_file_free(file), NuxStatus::Ok);
        }
    }

    #[test]
    fn editable_text_world_transform_includes_compound_parent_pose() {
        // Expected matrices are independent affine arithmetic, not runtime
        // decomposition. Nonuniform scale followed by rotation must retain
        // the entire basis; local Node.x/y alone cannot locate the editor.
        for (authored, expected) in [
            (
                [24.0, 24.0, 0.0, 1.0, 1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 1.0, 24.0, 24.0],
            ),
            (
                [24.0, 24.0, std::f32::consts::FRAC_PI_2, 2.0, 3.0, 7.0, 11.0],
                [0.0, 2.0, -3.0, 0.0, -9.0, 38.0],
            ),
            (
                [24.0, 264.0, 0.0, -2.0, 0.5, 7.0, 11.0],
                [-2.0, 0.0, 0.0, 0.5, 10.0, 269.5],
            ),
        ] {
            let bytes = fixture::transformed_compound_text_artboard(authored);
            unsafe {
                let mut file = ptr::null_mut();
                assert_eq!(
                    nux_file_import(
                        bytes.as_ptr(),
                        bytes.len(),
                        &NuxRenderCallbacks::default(),
                        &mut file
                    ),
                    NuxStatus::Ok
                );
                let mut instance = ptr::null_mut();
                assert_eq!(
                    nux_artboard_instance_new(file, 0, &mut instance),
                    NuxStatus::Ok
                );
                let mut player = ptr::null_mut();
                assert_eq!(nux_player_new_static(instance, &mut player), NuxStatus::Ok);
                let step = NuxPlayerStep {
                    struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
                    ..Default::default()
                };
                let mut result = ptr::null_mut();
                assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
                let name = "field/name";
                let mut geometry = NuxTextRunGeometry {
                    struct_size: std::mem::size_of::<NuxTextRunGeometry>() as u32,
                    ..Default::default()
                };
                assert_eq!(
                    nux_player_text_run_geometry(
                        player,
                        result,
                        NuxStringView {
                            data: name.as_ptr().cast(),
                            len: name.len()
                        },
                        &mut geometry
                    ),
                    NuxStatus::Ok
                );
                let actual = geometry.world_transform;
                for (actual, expected) in actual.into_iter().zip(expected) {
                    assert!(
                        (actual - expected).abs() < 0.0001,
                        "actual {actual}, expected {expected}; authored {authored:?}"
                    );
                }
                assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
                assert_eq!(nux_player_free(player), NuxStatus::Ok);
                assert_eq!(nux_artboard_instance_free(instance), NuxStatus::Ok);
                assert_eq!(nux_file_free(file), NuxStatus::Ok);
            }
        }
    }

    #[test]
    fn text_run_association_uses_the_presented_owner_and_rejects_ambiguity() {
        check_text_run_association(false);
        check_text_run_association(true);
    }

    fn check_text_run_association(compound: bool) {
        use nuxie::runtime::{core::CoreType, text::text_value_run::TextValueRun};
        let bytes = if compound {
            fixture::compound_semantic_text_artboard()
        } else {
            fixture::semantic_text_artboard()
        };
        let bytes = fixture::with_string_properties(bytes, &["editable", "duplicate", "duplicate"]);
        unsafe {
            let mut file = ptr::null_mut();
            assert_eq!(
                nux_file_import(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &mut file
                ),
                NuxStatus::Ok
            );
            let mut instance = ptr::null_mut();
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut instance),
                NuxStatus::Ok
            );
            let mut player = ptr::null_mut();
            assert_eq!(nux_player_new_static(instance, &mut player), NuxStatus::Ok);
            assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
            let capture = || {
                let step = NuxPlayerStep {
                    struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
                    ..Default::default()
                };
                let mut result = ptr::null_mut();
                assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
                let mut info = NuxPlayerSchedulingInfo {
                    struct_size: std::mem::size_of::<NuxPlayerSchedulingInfo>() as u32,
                    ..Default::default()
                };
                assert_eq!(
                    nux_player_step_result_scheduling(result, &mut info),
                    NuxStatus::Ok
                );
                assert_eq!(
                    nux_player_acknowledge_presented(player, info.render_revision),
                    NuxStatus::Ok
                );
                assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
                let mut snapshot = ptr::null_mut();
                assert_eq!(
                    nux_player_semantic_snapshot(player, &mut snapshot),
                    NuxStatus::Ok
                );
                snapshot
            };
            let first = capture();
            let artboard = (&(*player).artboard).instance.borrow().native_handle();
            let runs = artboard.with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .filter(|object| object.is_type_of(TextValueRun::TYPE_KEY))
                    .cloned()
                    .collect::<Vec<_>>()
            });
            assert_eq!(
                (&(*first).nodes).len(),
                if compound { 2 } else { 1 },
                "fixture has the authored field owner and optional inner text"
            );
            let id = (&(*first).nodes)
                .iter()
                .find(|node| node.label == "Name")
                .unwrap()
                .id;
            let data = artboard
                .with_artboard(|a| {
                    a.objects()
                        .iter()
                        .flatten()
                        .find(|object| object.is_type_of(SemanticData::TYPE_KEY))
                        .cloned()
                })
                .unwrap();
            let run = runs.first().unwrap();
            let name = "field/name".to_owned();
            let view = NuxStringView {
                data: name.as_ptr().cast(),
                len: name.len(),
            };
            let mut found = u32::MAX;
            assert_eq!(
                nux_player_semantic_node_for_text_run(player, first, view, &mut found),
                NuxStatus::NotFound
            );
            assert_eq!(found, 0);
            assert_eq!(nux_semantic_snapshot_free(first), NuxStatus::Ok);
            data.with_downcast_mut::<SemanticData, _>(|data| data.set_role(6));
            let mut snapshot = capture();
            assert_eq!(
                nux_player_semantic_node_for_text_run(player, snapshot, view, &mut found),
                NuxStatus::Ok
            );
            assert_eq!(found, id);
            let manager = artboard
                .with_artboard(|artboard| artboard.semantic_manager())
                .unwrap();
            let field = manager
                .with_semantic_manager(|manager| manager.node_by_id(id))
                .unwrap();
            let property =
                field_string_property(&field, "editable").expect("field-owned value resolves");
            assert_eq!(
                nuxie::runtime::generated::core_registry::CoreRegistry::get_string_handle(
                    &property, 246
                ),
                Some("editable value".into()),
            );
            assert!(matches!(
                field_string_property(&field, "absent"),
                Err(NuxStatus::NotFound)
            ));
            assert!(
                matches!(
                    field_string_property(&field, "field/name"),
                    Err(NuxStatus::NotFound)
                ),
                "a drawable TextValueRun is not an editable-value property"
            );
            assert!(matches!(
                field_string_property(&field, "duplicate"),
                Err(NuxStatus::InvalidArgument)
            ));
            assert!(matches!(
                field_string_property(&field, ""),
                Err(NuxStatus::InvalidArgument)
            ));
            assert!(matches!(
                field_string_property(&field, &"x".repeat(4097)),
                Err(NuxStatus::LimitExceeded)
            ));
            let string_view = |text: &str| NuxStringView {
                data: text.as_ptr().cast(),
                len: text.len(),
            };
            let property_name = string_view("editable");
            let mut length = 999;
            assert_eq!(
                nux_player_field_string_copy(
                    player,
                    snapshot,
                    id,
                    property_name,
                    ptr::null_mut(),
                    0,
                    &mut length
                ),
                NuxStatus::Ok
            );
            assert_eq!(length, "editable value".len());
            let mut short = [0xab; 2];
            assert_eq!(
                nux_player_field_string_copy(
                    player,
                    snapshot,
                    id,
                    property_name,
                    short.as_mut_ptr(),
                    short.len(),
                    &mut length
                ),
                NuxStatus::LimitExceeded
            );
            assert_eq!(short, [0xab; 2], "short reads cannot copy partial secrets");
            assert_eq!(
                nux_player_field_string_set(
                    player,
                    snapshot,
                    id,
                    string_view("absent"),
                    string_view("rejected")
                ),
                NuxStatus::NotFound
            );
            assert_eq!(
                nux_player_field_string_set(
                    player,
                    snapshot,
                    id,
                    property_name,
                    string_view("editable value")
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_validate_semantic_snapshot(player, snapshot),
                NuxStatus::Ok,
                "no-op writes preserve the presented revision"
            );
            assert_eq!(
                nux_player_field_string_set(
                    player,
                    snapshot,
                    id,
                    property_name,
                    string_view("秘密 🦊")
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_field_string_set(
                    player,
                    snapshot,
                    id,
                    property_name,
                    string_view("stale")
                ),
                NuxStatus::HandleMismatch
            );
            assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
            snapshot = capture();
            let mut copied = [0u8; 64];
            assert_eq!(
                nux_player_field_string_copy(
                    player,
                    snapshot,
                    id,
                    property_name,
                    copied.as_mut_ptr(),
                    copied.len(),
                    &mut length
                ),
                NuxStatus::Ok
            );
            assert_eq!(&copied[..length], "秘密 🦊".as_bytes());
            assert!(
                (&(*snapshot).nodes)
                    .iter()
                    .all(|node| !node.value.contains("秘密")),
                "non-rendering values do not enter ordinary semantic captures"
            );
            data.with_downcast_mut::<SemanticData, _>(|data| {
                data.set_state_flags(SemanticState::READ_ONLY.0)
            });
            assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
            snapshot = capture();
            assert_eq!(
                nux_player_field_string_set(
                    player,
                    snapshot,
                    id,
                    property_name,
                    string_view("forbidden")
                ),
                NuxStatus::NotFound
            );
            assert_eq!(
                nux_player_field_string_copy(
                    player,
                    snapshot,
                    id,
                    property_name,
                    copied.as_mut_ptr(),
                    copied.len(),
                    &mut length
                ),
                NuxStatus::Ok
            );
            assert_eq!(&copied[..length], "秘密 🦊".as_bytes());
            data.with_downcast_mut::<SemanticData, _>(|data| data.set_state_flags(0));
            assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
            snapshot = capture();
            if compound {
                let inner = artboard.with_artboard(|artboard| {
                    artboard
                        .objects()
                        .iter()
                        .flatten()
                        .find(|object| {
                            object.is_type_of(SemanticData::TYPE_KEY) && *object != &data
                        })
                        .cloned()
                        .unwrap()
                });
                inner.with_downcast_mut::<SemanticData, _>(|data| data.set_role(6));
                let ambiguous = capture();
                assert_eq!(
                    nux_player_semantic_node_for_text_run(player, ambiguous, view, &mut found),
                    NuxStatus::InvalidArgument,
                    "two semantic field owners on the exact text ancestry must not silently choose one"
                );
                assert_eq!(found, 0);
                assert_eq!(nux_semantic_snapshot_free(ambiguous), NuxStatus::Ok);
                assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
                inner.with_downcast_mut::<SemanticData, _>(|data| data.set_role(7));
                snapshot = capture();
                assert_eq!(
                    nux_player_semantic_node_for_text_run(player, snapshot, view, &mut found),
                    NuxStatus::Ok
                );
                assert_eq!(found, id);
            }
            let missing = "not an authored run";
            assert_eq!(
                nux_player_semantic_node_for_text_run(
                    player,
                    snapshot,
                    NuxStringView {
                        data: missing.as_ptr().cast(),
                        len: missing.len()
                    },
                    &mut found
                ),
                NuxStatus::NotFound
            );
            assert_eq!(found, 0);
            // A second occurrence cannot use this capture even when its file and names match.
            let mut other_instance = ptr::null_mut();
            let mut other = ptr::null_mut();
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut other_instance),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_new_static(other_instance, &mut other),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_semantic_node_for_text_run(other, snapshot, view, &mut found),
                NuxStatus::HandleMismatch
            );
            nux_player_free(other);
            nux_artboard_instance_free(other_instance);
            // Ambiguous authored names fail instead of silently associating the first field.
            let duplicate = runs
                .iter()
                .find(|candidate| *candidate != run)
                .expect("fixture has multiple text runs");
            assert!(
                nuxie::runtime::generated::core_registry::CoreRegistry::set_string_handle(
                    duplicate,
                    nuxie::runtime::generated::component_base::ComponentBase::NAME_PROPERTY_KEY
                        .into(),
                    name.clone()
                )
            );
            assert_eq!(
                nux_player_semantic_node_for_text_run(player, snapshot, view, &mut found),
                NuxStatus::InvalidArgument
            );
            assert_eq!(found, 0);
            let changed_text = b"new private field value";
            let mutation = NuxTextRunMutation {
                name: view,
                text: NuxByteView {
                    data: changed_text.as_ptr(),
                    len: changed_text.len(),
                },
            };
            let batch = NuxTextRunMutationBatch {
                mutations: &mutation,
                mutation_count: 1,
                ..Default::default()
            };
            assert_eq!(
                nux_artboard_instance_set_text_runs(instance, &batch, ptr::null_mut()),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_semantic_node_for_text_run(player, snapshot, view, &mut found),
                NuxStatus::HandleMismatch
            );
            assert_eq!(found, 0);
            assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
            nux_player_free(player);
            nux_artboard_instance_free(instance);
            nux_file_free(file);
        }
    }

    #[test]
    fn capture_rejects_over_budget_clip_and_recovers_without_partial_handle() {
        use nuxie::runtime::{
            generated::{core_registry::CoreRegistry, layout_component_base::LayoutComponentBase},
            math::path_types::PathDirection,
        };
        let root = std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
        let bytes = std::fs::read(
            std::path::PathBuf::from(root).join("tests/unit_tests/assets/semantic/simpsons.riv"),
        )
        .unwrap();
        let mut file = ptr::null_mut();
        let mut instance = ptr::null_mut();
        let mut player = ptr::null_mut();
        unsafe {
            assert_eq!(
                nux_file_import(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &mut file
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut instance),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_new_static(instance, &mut player), NuxStatus::Ok);
            assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
            let native = (&(*player).artboard).instance.borrow().native_handle();
            assert!(CoreRegistry::set_bool_handle(
                &native.core_handle(),
                LayoutComponentBase::CLIP_PROPERTY_KEY.into(),
                true
            ));
            let step = NuxPlayerStep {
                struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
                ..Default::default()
            };
            let mut result = ptr::null_mut();
            assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
            let mut scheduling = NuxPlayerSchedulingInfo {
                struct_size: std::mem::size_of::<NuxPlayerSchedulingInfo>() as u32,
                ..Default::default()
            };
            assert_eq!(
                nux_player_step_result_scheduling(result, &mut scheduling),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_acknowledge_presented(player, scheduling.render_revision),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
            let mut snapshot = ptr::null_mut();
            assert_eq!(
                nux_player_semantic_snapshot(player, &mut snapshot),
                NuxStatus::Ok
            );
            assert!(
                !(*snapshot).nodes.is_empty(),
                "fixture must expose semantic controls"
            );
            assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
            // Replace the already-built rendered clip with an equivalent, valid
            // nonzero path that exceeds the semantic computation budget.
            native.with_artboard_mut(|artboard| {
                let bounds = artboard.bounds();
                let path = artboard.local_path().unwrap();
                path.rewind();
                for _ in 0..5000 {
                    path.add_rect(bounds, PathDirection::Clockwise);
                }
            });
            snapshot = ptr::dangling_mut();
            assert_eq!(
                nux_player_semantic_snapshot(player, &mut snapshot),
                NuxStatus::LimitExceeded
            );
            assert!(
                snapshot.is_null(),
                "failure must never publish a partial capture"
            );
            native.with_artboard_mut(|artboard| {
                let bounds = artboard.bounds();
                let path = artboard.local_path().unwrap();
                path.rewind();
                path.add_rect(bounds, PathDirection::Clockwise);
            });
            assert_eq!(
                nux_player_semantic_snapshot(player, &mut snapshot),
                NuxStatus::Ok
            );
            assert!(!(*snapshot).nodes.is_empty());
            assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
            assert_eq!(nux_player_free(player), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_free(instance), NuxStatus::Ok);
            assert_eq!(nux_file_free(file), NuxStatus::Ok);
        }
    }

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
            actions: vec![0],
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
        assert_eq!(wrong_thread, NuxStatus::WrongThread);
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
