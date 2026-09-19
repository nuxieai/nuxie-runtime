use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::pfunc::Pfunc;

#[allow(non_snake_case)]
#[export_name = "luaur_lua_d_rawrunprotected_mut"]
pub unsafe fn lua_d_rawrunprotected_mut(
    L: *mut lua_State,
    f: Pfunc,
    ud: *mut core::ffi::c_void,
) -> i32 {
    crate::functions::lua_d_rawrunprotected_ldo::luaD_rawrunprotected(L, f, ud)
}
