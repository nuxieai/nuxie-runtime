use crate::macros::api_checknelems::api_checknelems;
use crate::type_aliases::lua_state::lua_State;

use crate::enums::lua_status::lua_Status;

use crate::functions::lua_d_throw_ldo::lua_d_throw;

// lapi.cpp — l_noret lua_error(lua_State* L) { api_checknelems(L, 1); luaD_throw(L, LUA_ERRRUN); }
#[allow(non_snake_case)]
pub unsafe fn lua_error(L: *mut lua_State) -> ! {
    api_checknelems!(L, 1);
    lua_d_throw(L, lua_Status::LUA_ERRRUN as i32)
}
