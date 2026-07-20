//! Node: `cxx:Function:Luau.VM:VM/src/ldo.cpp:162:lua_d_throw`
//! Source: `VM/src/ldo.cpp` (ldo.cpp:162-165, hand-ported; C++-exceptions build flavor,
//! matching the catch_unwind-based luaD_rawrunprotected)

use crate::records::lua_exception::lua_exception;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn luaD_throw(l: *mut lua_State, errcode: core::ffi::c_int) -> ! {
    std::panic::panic_any(lua_exception::lua_exception(l, errcode));
}

#[allow(unused_imports)]
pub use luaD_throw as lua_d_throw;
