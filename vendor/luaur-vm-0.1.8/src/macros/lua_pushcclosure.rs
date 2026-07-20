use crate::functions::lua_pushcclosurek::lua_pushcclosurek;
use crate::type_aliases::lua_c_function::lua_CFunction;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_upper_case_globals)]
pub const lua_pushcclosure: unsafe fn(
    *mut lua_State,
    lua_CFunction,
    *const core::ffi::c_char,
    core::ffi::c_int,
) = |l, f, debugname, nup| unsafe {
    lua_pushcclosurek(l, f, debugname, nup, None);
};

#[allow(non_upper_case_globals)]
pub const LUA_PUSHCCLOSURE: unsafe fn(
    *mut lua_State,
    lua_CFunction,
    *const core::ffi::c_char,
    core::ffi::c_int,
) = lua_pushcclosure;
