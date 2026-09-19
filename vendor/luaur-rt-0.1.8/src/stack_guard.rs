use crate::sys::{c_int, lua_State, lua_gettop, lua_pop};

/// Restore an embedding call's stack when a returned error exits early.
/// The call may push and consume its own values, but not values below base.
pub(crate) struct StackGuard {
    state: *mut lua_State,
    base: c_int,
}

impl StackGuard {
    /// The state must remain live, on this thread, until the guard is dropped.
    pub(crate) unsafe fn new(state: *mut lua_State) -> Self {
        Self {
            state,
            base: unsafe { lua_gettop(state) },
        }
    }
}

impl Drop for StackGuard {
    fn drop(&mut self) {
        unsafe {
            let top = lua_gettop(self.state);
            assert!(
                top >= self.base,
                "embedding call consumed its caller's stack"
            );
            // Negative-index settop only shrinks; it cannot allocate or invoke
            // guest code. A failure here would violate the VM API invariant.
            lua_pop(self.state, top - self.base).expect("shrinking the stack cannot fail");
        }
    }
}
