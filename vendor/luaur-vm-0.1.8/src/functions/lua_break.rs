use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_throw_ldo::luaD_throw;
use crate::functions::lua_g_pusherror::lua_g_pusherror;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn lua_break(l: *mut lua_State) -> c_int {
    if (*l).nCcalls > (*l).baseCcalls {
        lua_g_pusherror(
            l,
            c"attempt to break across metamethod/C-call boundary".as_ptr(),
        );
        luaD_throw(l, lua_Status::LUA_ERRRUN as c_int);
    }

    (*l).status = lua_Status::LUA_BREAK as u8;
    -1
}
