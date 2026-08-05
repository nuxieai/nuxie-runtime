use crate::functions::lua_l_checknumber::lua_l_checknumber;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::type_aliases::lua_state::lua_State;

// Fallback for the LBF_RIVE_FROUND fastcall.
#[export_name = "luaur_math_fround"]
pub unsafe fn math_fround(l: *mut lua_State) -> core::ffi::c_int {
    lua_pushnumber(l, (lua_l_checknumber(l, 1) as f32) as f64);
    1
}
