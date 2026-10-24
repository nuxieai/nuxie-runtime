//! Published live-move plans. Mutations run atomically between player frames.
use crate::*;
// This module decodes and applies requests only. The existing player step
// boundary owns guards, transactions, effects, publication and failure policy.
pub(super) enum Request {
    Begin(NuxStringView, *const NuxStringView),
    End(NuxStringView, *const NuxStringView),
    Row(NuxStringView, u32, u8),
}
use nuxie::page_player::Operation as Prepared;
impl Request {
    pub(super) fn prepare(self) -> Result<Prepared, NuxStatus> {
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
                Prepared::Begin(incoming, name)
            }
            Self::End(incoming, name) => {
                let (incoming, name) = names(incoming, name)?;
                Prepared::End(incoming, name)
            }
            Self::Row(list, row, collapsed) => {
                if collapsed > 1 || row > i32::MAX as u32 {
                    return Err(NuxStatus::InvalidArgument);
                }
                Prepared::Row(
                    with_utf8_view(list, str::to_owned)?,
                    row as i32,
                    collapsed != 0,
                )
            }
        })
    }
}
/// Take off units from a published LiveStep. incoming_screen names the mounted,
/// expanded Screen artboard. A NULL step_name performs immediate coming home
/// for a cut. Returns INVALID_ARGUMENT for an unknown step, incompatible slots,
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
/// NULL step_name. The outgoing version of a two-version unit returns home.
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
