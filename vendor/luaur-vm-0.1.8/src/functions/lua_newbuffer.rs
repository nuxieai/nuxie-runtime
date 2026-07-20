//! Node: `cxx:Function:Luau.VM:VM/src/lapi.cpp:1477:lua_newbuffer`
//!
//! `lua_newbuffer` — allocate a managed buffer object of `sz` bytes, push it on
//! the stack, and return a pointer to its data. Runs a GC step and the thread
//! write-barrier first, exactly like the C++ public API.

use crate::functions::lua_b_newbuffer::lua_b_newbuffer;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::setbufvalue::setbufvalue;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub fn lua_newbuffer(L: *mut lua_State, sz: usize) -> *mut core::ffi::c_void {
    unsafe {
        luaC_checkGC!(L);
        lua_c_threadbarrier_lapi(L);
        let b = lua_b_newbuffer(L, sz);
        setbufvalue!(L, (*L).top, b);
        api_incr_top!(L);
        (*b).data.as_mut_ptr() as *mut core::ffi::c_void
    }
}
