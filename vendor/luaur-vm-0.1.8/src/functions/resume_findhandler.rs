use crate::macros::lua_callinfo_handle::LUA_CALLINFO_HANDLE;
use crate::records::call_info::CallInfo;
use crate::records::lua_state::lua_State;

pub(crate) unsafe fn resume_findhandler(L: *mut lua_State) -> *mut CallInfo {
    let mut ci = (*L).ci;

    while ci > (*L).base_ci {
        if ((*ci).flags & LUA_CALLINFO_HANDLE as u32) != 0 {
            return ci;
        }

        ci = ci.offset(-1);
    }

    core::ptr::null_mut()
}
