use crate::enums::lua_co_status::lua_CoStatus;
use crate::enums::lua_status::lua_Status;
use crate::macros::api_check::api_check;
use crate::records::lua_state::lua_State;

#[allow(non_snake_case)]
pub fn lua_costatus(l: *mut lua_State, co: *mut lua_State) -> core::ffi::c_int {
    unsafe {
        api_check!(l, !co.is_null());
        api_check!(l, (*l).global == (*co).global);

        if co == l {
            return lua_CoStatus::LUA_CORUN as core::ffi::c_int;
        }
        // Nuxie: the main thread is never a coroutine another thread can
        // resume or close. Outside a call it sits at frame depth zero, which
        // reads as suspended below, yet any other running thread was resumed
        // from it (directly or through a chain), and the host may hold values
        // in its stack slots meanwhile: it is normal.
        if co == (*(*co).global).mainthread {
            return lua_CoStatus::LUA_CONOR as core::ffi::c_int;
        }
        if (*co).status as i32 == lua_Status::LUA_YIELD as i32 {
            return lua_CoStatus::LUA_COSUS as core::ffi::c_int;
        }
        if (*co).status as i32 == lua_Status::LUA_BREAK as i32 {
            return lua_CoStatus::LUA_CONOR as core::ffi::c_int;
        }
        if (*co).status != 0 {
            return lua_CoStatus::LUA_COERR as core::ffi::c_int;
        }
        if (*co).ci != (*co).base_ci {
            return lua_CoStatus::LUA_CONOR as core::ffi::c_int;
        }
        if (*co).top == (*co).base {
            return lua_CoStatus::LUA_COFIN as core::ffi::c_int;
        }
        lua_CoStatus::LUA_COSUS as core::ffi::c_int
    }
}
