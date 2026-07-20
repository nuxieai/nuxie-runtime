//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:485:shrinkstackprotected`
//! Source: `VM/src/lgc.cpp` (lgc.cpp:485-498, hand-ported)

use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_rawrunprotected_ldo_alt_b::lua_d_rawrunprotected_mut;
use crate::functions::shrinkstack::shrinkstack;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

// C++ uses a local `struct CallContext { static void run(...) }`; a local fn is
// the Rust equivalent of that protected-call trampoline.
unsafe fn run(l: *mut lua_State, _ud: *mut core::ffi::c_void) {
    shrinkstack(l);
}

#[allow(non_snake_case)]
pub(crate) unsafe fn shrinkstackprotected(l: *mut lua_State) {
    // the resize call can fail on exception, in which case we will continue with original size
    let status = lua_d_rawrunprotected_mut(l, Some(run), core::ptr::null_mut());
    LUAU_ASSERT!(status == lua_Status::LUA_OK as i32 || status == lua_Status::LUA_ERRMEM as i32);
}
