use crate::functions::lua_d_reallocstack::lua_d_reallocstack;
use crate::macros::condhardstacktests::condhardstacktests;
use crate::macros::extra_stack::EXTRA_STACK;
use crate::macros::getgrownstacksize::getgrownstacksize;
use crate::macros::stacklimitreached::stacklimitreached;

use crate::records::lua_state::LuaState;
use core::ffi::c_int;

#[allow(non_snake_case)]
#[inline]
pub fn luaD_checkstackfornewci(L: *mut LuaState, n: c_int) {
    unsafe {
        if stacklimitreached(L, n) {
            lua_d_reallocstack(L, getgrownstacksize(L, n), 1);
        } else {
            condhardstacktests!(lua_d_reallocstack(L, (*L).stacksize - EXTRA_STACK, 1));
        }
    }
}
