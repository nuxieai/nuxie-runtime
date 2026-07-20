use crate::functions::lua_pushnil::lua_pushnil;
use crate::functions::lua_pushthread::lua_pushthread;
use crate::type_aliases::lua_state::lua_State;

pub(crate) unsafe fn corunning(L: *mut lua_State) -> core::ffi::c_int {
    if lua_pushthread(L) != 0 {
        lua_pushnil(L); // main thread is not a coroutine
    }
    1
}
