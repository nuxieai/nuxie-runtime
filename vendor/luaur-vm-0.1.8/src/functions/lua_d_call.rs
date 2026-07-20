use crate::functions::lua_d_callint::lua_d_callint;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

// plain `unsafe fn` (not extern "C"): Lua errors unwind through here via
// panic/catch_unwind, and extern "C" frames abort on unwind
pub unsafe fn lua_d_call(L: *mut lua_State, func: StkId, nresults: core::ffi::c_int) {
    lua_d_callint(L, func, nresults, false);
}
