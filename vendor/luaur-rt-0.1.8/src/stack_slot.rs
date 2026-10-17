//! Identity of a stack slot that host code holds across calls.
//!
//! A positive stack index is relative to the call frame current when it is
//! used, so a handle that holds a slot (a looked-up field, a held userdata, a
//! callback's arguments) records the frame it was taken in and refuses use
//! from any other: a scoped callback that captured it runs in a nested frame,
//! where the same index names another value.

use crate::error::{Error, Result};
use crate::sys::{c_int, lua_State};

/// A slot of the frame current on `state` when it was taken.
#[derive(Clone, Copy)]
pub(crate) struct FrameSlot {
    state: *mut lua_State,
    frame: usize,
    slot: c_int,
}

impl FrameSlot {
    /// `slot` (a positive index) of the frame current on `state` now.
    pub(crate) fn current(state: *mut lua_State, slot: c_int) -> Self {
        Self {
            state,
            frame: unsafe { frame_of(state) },
            slot,
        }
    }

    /// The slot's index, while `state`'s current frame is the one it was
    /// taken in.
    pub(crate) fn index(&self) -> Result<c_int> {
        if unsafe { frame_of(self.state) } != self.frame {
            return Err(Error::runtime(
                "luaur-rt: a stack-held value was used outside the call frame that holds it",
            ));
        }
        Ok(self.slot)
    }

    /// The index `offset` slots above this one, in the same frame.
    pub(crate) fn above(&self, offset: c_int) -> Self {
        Self {
            slot: self.slot + offset,
            ..*self
        }
    }
}

/// The current call frame of `state`: the byte offset of its `CallInfo` in
/// the frame array, stable when the array is reallocated and distinct for
/// every nesting depth.
unsafe fn frame_of(state: *mut lua_State) -> usize {
    unsafe { ((*state).ci as usize).wrapping_sub((*state).base_ci as usize) }
}
