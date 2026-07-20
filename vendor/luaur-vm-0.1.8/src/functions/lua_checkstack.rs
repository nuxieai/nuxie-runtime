use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_growstack::lua_d_growstack;
use crate::functions::lua_d_rawrunprotected_ldo::lua_d_rawrunprotected;
use crate::functions::lua_d_reallocstack::lua_d_reallocstack;
use crate::macros::api_check::api_check;
use crate::macros::condhardstacktests::condhardstacktests;
use crate::macros::expandstacklimit::expandstacklimit;
use crate::macros::extra_stack::EXTRA_STACK;
use crate::macros::luai_maxcstack::LUAI_MAXCSTACK;
use crate::macros::stacklimitreached::stacklimitreached;
use crate::records::call_context_lapi::CallContext;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_int, c_void};

unsafe fn call_context_run(L: *mut lua_State, ud: *mut c_void) {
    let ctx = ud as *mut CallContext;
    lua_d_growstack(L, (*ctx).size);
}

#[allow(non_snake_case)]
pub unsafe fn lua_checkstack(L: *mut lua_State, size: c_int) -> c_int {
    api_check!(L, size >= 0);

    let mut res = 1;
    if size > LUAI_MAXCSTACK || ((*L).top.offset_from((*L).base) as c_int + size) > LUAI_MAXCSTACK {
        res = 0; // stack overflow
    } else if size > 0 {
        if stacklimitreached(L, size) {
            let mut ctx = CallContext { size };
            // there could be no memory to extend the stack
            if lua_d_rawrunprotected(
                L,
                Some(call_context_run),
                core::ptr::addr_of_mut!(ctx) as *mut c_void,
            ) != lua_Status::LUA_OK as c_int
            {
                return 0;
            }
        } else {
            condhardstacktests!(lua_d_reallocstack(L, (*L).stacksize - EXTRA_STACK, 0));
        }

        expandstacklimit!(L, (*L).top.add(size as usize));
    }
    res
}
