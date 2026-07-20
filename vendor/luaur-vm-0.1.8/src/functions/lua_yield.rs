use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_throw_ldo::luaD_throw;
use crate::functions::lua_g_pusherror::lua_g_pusherror;
use crate::macros::api_check::api_check;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn lua_yield(l: *mut lua_State, nresults: c_int) -> c_int {
    api_check!(l, nresults >= 0);
    api_check!(l, nresults as isize <= (*l).top.offset_from((*l).base));

    if (*l).nCcalls > (*l).baseCcalls {
        lua_g_pusherror(
            l,
            c"attempt to yield across metamethod/C-call boundary".as_ptr(),
        );
        luaD_throw(l, lua_Status::LUA_ERRRUN as c_int);
    }

    (*l).base = (*l).top.offset(-(nresults as isize));
    (*l).status = lua_Status::LUA_YIELD as u8;
    -1
}
