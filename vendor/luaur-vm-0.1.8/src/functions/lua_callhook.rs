use crate::records::lua_state::lua_State;
use crate::type_aliases::lua_hook::LuaHook;

pub unsafe fn lua_callhook(l: *mut lua_State, hook: LuaHook, userdata: *mut core::ffi::c_void) {
    crate::macros::api_check::api_check!(l, hook.is_some());
    crate::macros::api_check::api_check!(l, (*l).ci != (*l).base_ci);
    crate::functions::luau_callhook::luau_callhook(l, hook, userdata);
}
