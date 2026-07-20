use crate::functions::auxresumecont::auxresumecont;
use crate::functions::coresumefinish::coresumefinish;
use crate::functions::interrupt_thread::interrupt_thread;
use crate::functions::lua_tothread::lua_tothread;
use crate::macros::lua_l_argexpected::luaL_argexpected;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn coresumecont(l: *mut lua_State, status: core::ffi::c_int) -> core::ffi::c_int {
    let co = lua_tothread(l, 1);
    luaL_argexpected!(l, !co.is_null(), 1, "thread");

    // if coroutine still hasn't yielded after the break, break current thread again
    if (*co).status == crate::enums::lua_status::lua_Status::LUA_BREAK as u8 {
        return interrupt_thread(l, co);
    }

    let r = auxresumecont(l, co);
    coresumefinish(l, r)
}
