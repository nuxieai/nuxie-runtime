use crate::functions::auxresume::auxresume;
use crate::functions::coresumefinish::coresumefinish;
use crate::functions::interrupt_thread::interrupt_thread;
use crate::functions::lua_tothread::lua_tothread;
use crate::macros::cast_int::cast_int;
use crate::macros::co_status_break::CO_STATUS_BREAK;
use crate::macros::lua_l_argexpected::luaL_argexpected;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

pub unsafe fn coresumey(L: *mut lua_State) -> c_int {
    let co = lua_tothread(L, 1);
    luaL_argexpected!(L, !co.is_null(), 1, "thread");
    let narg = cast_int!((*L).top.offset_from((*L).base)) - 1;
    let r = auxresume(L, co, narg);

    if r == CO_STATUS_BREAK {
        return interrupt_thread(L, co);
    }

    coresumefinish(L, r)
}
