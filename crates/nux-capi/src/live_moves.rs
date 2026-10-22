use crate::*;

fn with_moves(
    player: *mut NuxPlayer,
    operation: impl FnOnce(
        &mut nuxie::live_move::LiveMoves,
        &nuxie::RuntimeArtboardInstanceHandle,
    ) -> bool,
) -> NuxStatus {
    let _call = match enter_handle(player, HandleKind::Player) {
        Ok(guard) => guard,
        Err(status) => return status,
    };
    let Some(player) = (unsafe { player.as_ref() }) else {
        return NuxStatus::NullArgument;
    };
    let _occurrence = match enter_occurrence(&player.artboard) {
        Ok(guard) => guard,
        Err(status) => return status,
    };
    if player.artboard.poisoned.get() {
        return NuxStatus::RuntimeError;
    }
    let Ok(artboard) = player.artboard.instance.try_borrow_mut() else {
        return NuxStatus::ReentrantCall;
    };
    let Ok(mut moves) = player.live_moves.try_borrow_mut() else {
        return NuxStatus::ReentrantCall;
    };
    if !operation(&mut moves, &artboard.native_handle()) {
        return NuxStatus::InvalidArgument;
    }
    drop(artboard);
    player.artboard.commit_runtime_change_or_poison(true)
}

/// Install live moves JSON and empty mounted flight slots.
/// Returns INVALID_ARGUMENT for malformed JSON or replacement during a step.
/// Input is length-delimited UTF-8, bounded to 1 MiB. Same thread and reentrancy
/// rules as nux_player_step apply.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_set_moves(
    player: *mut NuxPlayer,
    json: NuxStringView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if json.len > 1024 * 1024 {
            return NuxStatus::LimitExceeded;
        }
        match with_utf8_view(json, |json| {
            with_moves(player, |moves, root| {
                moves
                    .set_moves(json)
                    .and_then(|_| moves.prepare(root))
                    .is_ok()
            })
        }) {
            Ok(status) | Err(status) => status,
        }
    })
}

fn step_boundary(
    player: *mut NuxPlayer,
    incoming: NuxStringView,
    transition: *const NuxStringView,
    end: bool,
) -> NuxStatus {
    let incoming = match with_utf8_view(incoming, str::to_owned) {
        Ok(value) => value,
        Err(status) => return status,
    };
    let transition = if transition.is_null() {
        None
    } else {
        match with_utf8_view(unsafe { *transition }, str::to_owned) {
            Ok(value) => Some(value),
            Err(status) => return status,
        }
    };
    with_moves(player, |moves, root| {
        if end {
            moves
                .step_end(root, &incoming, transition.as_deref())
                .is_ok()
        } else {
            moves
                .step_begin(root, &incoming, transition.as_deref())
                .is_ok()
        }
    })
}

/// Show a retained Screen row and take off the transition's live units.
/// A NULL transition_id performs immediate coming-home moves only.
/// Returns INVALID_ARGUMENT for missing/ambiguous slots, incompatible units,
/// overlapping flights, or an already active step. Rows must be mounted first.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_step_begin(
    player: *mut NuxPlayer,
    incoming_screen_id: NuxStringView,
    transition_id: *const NuxStringView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        step_boundary(player, incoming_screen_id, transition_id, false)
    })
}

/// Land all live flights. Both identifiers must match the active step.
/// A NULL transition_id matches a step begun without a transition.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_step_end(
    player: *mut NuxPlayer,
    incoming_screen_id: NuxStringView,
    transition_id: *const NuxStringView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        step_boundary(player, incoming_screen_id, transition_id, true)
    })
}

/// Set collapse on one mounted row of a named non-virtual ArtboardComponentList.
/// Collapsed rows retain their instances and VM values but skip playback,
/// layout, drawing and hits. collapsed must be 0 or 1. Unknown rows or lists
/// return INVALID_ARGUMENT. This does not reorder or remove a row.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_set_row_collapsed(
    player: *mut NuxPlayer,
    list_name: NuxStringView,
    row_index: u32,
    collapsed: u8,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if collapsed > 1 || row_index > i32::MAX as u32 {
            return NuxStatus::InvalidArgument;
        }
        match with_utf8_view(list_name, |name| {
            with_moves(player, |_, root| {
                let list = root.with_artboard(|b| {
                    b.find_handle::<nuxie::runtime::artboard_component_list::ArtboardComponentList>(
                        name,
                    )
                });
                list.is_some_and(|list| {
                    nuxie::live_move::set_row_collapsed(&list, row_index as i32, collapsed != 0)
                        .is_ok()
                })
            })
        }) {
            Ok(status) | Err(status) => status,
        }
    })
}
