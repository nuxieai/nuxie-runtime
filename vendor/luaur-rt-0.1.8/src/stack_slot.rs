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
        Self::at(state, slot).0
    }

    /// [`FrameSlot::current`] for a slot whose value the host holds there
    /// across calls (it may run Lua meanwhile), once
    /// [`FrameSlot::require_active`] allows it.
    pub(crate) fn held(state: *mut lua_State, slot: c_int) -> Result<Self> {
        Self::require_active(state)?;
        Ok(Self::current(state, slot))
    }

    /// Whether `state` may hold values in its slots across calls. Only an
    /// active thread can: the main thread, or a coroutine that is running or
    /// has resumed another. Neither can be reset, closed or resumed while
    /// the holder runs, so a slot keeps its value rooted. A suspended,
    /// finished or failed coroutine (reached through a handle captured while
    /// it ran) could be reset under the holder, which would free the value,
    /// and a suspended one cannot run protected calls: refused.
    pub(crate) fn require_active(state: *mut lua_State) -> Result<()> {
        if !Self::at(state, 0).1 {
            return Err(Error::runtime(
                "luaur-rt: a stack-held value needs the main thread or an active coroutine",
            ));
        }
        Ok(())
    }

    fn at(state: *mut lua_State, slot: c_int) -> (Self, bool) {
        let frame = unsafe { frame_of(state) };
        (
            Self {
                state,
                frame: frame.offset,
                slot,
            },
            frame.active,
        )
    }

    /// The slot's index, while `state`'s current frame is the one it was
    /// taken in.
    pub(crate) fn index(&self) -> Result<c_int> {
        if unsafe { frame_of(self.state) }.offset != self.frame {
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

/// Where `state` stands now.
struct Frame {
    /// The byte offset of its current `CallInfo` in the frame array, stable
    /// when the array is reallocated and distinct for every nesting depth.
    offset: usize,
    /// Whether it is the main thread or a coroutine inside a call (running,
    /// or normal: it resumed the running thread). `lua_costatus` reads the
    /// same fields: a yielded or failed coroutine has a non-OK status, and
    /// an unstarted or finished one sits at its base frame.
    active: bool,
}

unsafe fn frame_of(state: *mut lua_State) -> Frame {
    unsafe {
        let offset = ((*state).ci as usize).wrapping_sub((*state).base_ci as usize);
        let main = state == (*(*state).global).mainthread;
        let calling =
            (*state).status == luaur_vm::enums::lua_status::lua_Status::LUA_OK as u8 && offset != 0;
        Frame {
            offset,
            active: main || calling,
        }
    }
}
