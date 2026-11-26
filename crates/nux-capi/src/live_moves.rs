//! Published live-move plans. Mutations run atomically between player frames.
use crate::*;
// This module decodes and applies requests only. The existing player step
// boundary owns guards, transactions, effects, publication and failure policy.
pub(super) enum Request {
    Begin(NuxStringView, *const NuxStringView),
    End(NuxStringView, *const NuxStringView),
    Row(NuxStringView, u32, u8),
    KeyedRow(NuxStringView, NuxByteView, u8),
}
use nuxie::page_player::Operation;
impl Request {
    pub(super) fn prepare(self) -> Result<Operation, NuxStatus> {
        fn names(
            incoming: NuxStringView,
            name: *const NuxStringView,
        ) -> Result<(String, Option<String>), NuxStatus> {
            Ok((
                with_utf8_view(incoming, str::to_owned)?,
                if name.is_null() {
                    None
                } else {
                    Some(with_utf8_view(unsafe { *name }, str::to_owned)?)
                },
            ))
        }
        Ok(match self {
            Self::Begin(incoming, name) => {
                let (incoming, name) = names(incoming, name)?;
                Operation::Begin(incoming, name)
            }
            Self::End(incoming, name) => {
                let (incoming, name) = names(incoming, name)?;
                Operation::End(incoming, name)
            }
            Self::Row(list, row, collapsed) => {
                if collapsed > 1 || row > i32::MAX as u32 {
                    return Err(NuxStatus::InvalidArgument);
                }
                Operation::Row(
                    with_utf8_view(list, str::to_owned)?,
                    row as i32,
                    collapsed != 0,
                )
            }
            Self::KeyedRow(list, key, collapsed) => {
                if collapsed > 1 {
                    return Err(NuxStatus::InvalidArgument);
                }
                if key.data.is_null() && key.len != 0 {
                    return Err(NuxStatus::NullArgument);
                }
                let key = if key.len == 0 {
                    Vec::new()
                } else {
                    unsafe { std::slice::from_raw_parts(key.data, key.len) }.to_vec()
                };
                Operation::KeyedRow(with_utf8_view(list, str::to_owned)?, key, collapsed != 0)
            }
        })
    }
}
/// Take off the units of a published LiveStep. incoming_screen names the
/// mounted, expanded Screen artboard. A NULL step_name plays no flights.
/// Returns INVALID_ARGUMENT for an unknown step, unmatched elements,
/// collapsed incoming Screen, or an already active step. Uses the player's
/// normal thread, occurrence, reentrancy and render-revision guards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_step_begin(
    player: *mut NuxPlayer,
    incoming_screen: NuxStringView,
    step_name: *const NuxStringView,
) -> NuxStatus {
    player_live_move_body(player, Request::Begin(incoming_screen, step_name))
}
/// Land a published step. Both arguments must match step_begin, including a
/// NULL step_name. Land every flight: each copy on its own Screen, and
/// a transferred unit in the arriving placement, whose departure placement gets
/// a fresh instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_step_end(
    player: *mut NuxPlayer,
    incoming_screen: NuxStringView,
    step_name: *const NuxStringView,
) -> NuxStatus {
    player_live_move_body(player, Request::End(incoming_screen, step_name))
}
/// Collapse or wake one mounted row of a named non-virtual list. collapsed is
/// 0 or 1. Retains playback, view models, video and native field identities.
/// Unknown lists, rows and virtualised lists return INVALID_ARGUMENT.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_set_row_collapsed(
    player: *mut NuxPlayer,
    list_name: NuxStringView,
    row_index: u32,
    collapsed: u8,
) -> NuxStatus {
    player_live_move_body(player, Request::Row(list_name, row_index, collapsed))
}
/// Collapse or wake the row mounted by `key` on a named keyed list that is
/// the page's rows (`isPageRows`), as nux_player_set_row_collapsed does for
/// a row by its index. collapsed is 0 or 1. The key's bytes are compared
/// only for equality. Retains the row's playback, state and place. An
/// unknown list, a list that is not keyed or not the page's, a key not
/// mounted and a collapsed value past 1 return INVALID_ARGUMENT; a NULL key
/// with a length returns NULL_ARGUMENT.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_set_keyed_row_collapsed(
    player: *mut NuxPlayer,
    list_name: NuxStringView,
    key: NuxByteView,
    collapsed: u8,
) -> NuxStatus {
    player_live_move_body(player, Request::KeyedRow(list_name, key, collapsed))
}
