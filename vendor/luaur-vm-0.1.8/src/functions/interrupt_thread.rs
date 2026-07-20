use crate::functions::lua_break::lua_break;
use crate::functions::luau_callhook::luau_callhook;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn interrupt_thread(l: *mut lua_State, co: *mut lua_State) -> core::ffi::c_int {
    let global = (*l).global;
    let debuginterrupt = (*global).cb.debuginterrupt;
    if debuginterrupt.is_some() {
        luau_callhook(l, debuginterrupt, co as *mut core::ffi::c_void);
    }

    lua_break(l)
}
