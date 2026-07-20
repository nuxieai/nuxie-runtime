use crate::enums::lua_status::lua_Status;
use crate::functions::auxresumecont::auxresumecont;
use crate::functions::auxwrapfinish::auxwrapfinish;
use crate::functions::interrupt_thread::interrupt_thread;
use crate::functions::lua_tothread::lua_tothread;
use crate::macros::lua_upvalueindex::lua_upvalueindex;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn auxwrapcont(l: *mut lua_State, status: core::ffi::c_int) -> core::ffi::c_int {
    let co = lua_tothread(l, lua_upvalueindex(1));

    if (*co).status == lua_Status::LUA_BREAK as u8 {
        return interrupt_thread(l, co);
    }

    let r = auxresumecont(l, co);
    auxwrapfinish(l, r)
}
