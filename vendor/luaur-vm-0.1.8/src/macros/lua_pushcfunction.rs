use crate::functions::lua_pushcclosurek::lua_pushcclosurek;
use crate::type_aliases::lua_c_function::lua_CFunction;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_upper_case_globals)]
pub const LUA_PUSHCFUNCTION: unsafe fn(*mut lua_State, lua_CFunction, *const core::ffi::c_char) =
    |l, f, debugname| unsafe {
        lua_pushcclosurek(l, f, debugname, 0, None);
    };
