use crate::functions::lua_clock::lua_clock;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::type_aliases::lua_state::lua_State;

pub(crate) unsafe fn os_clock(L: *mut lua_State) -> i32 {
    lua_pushnumber(L, lua_clock());
    1
}
