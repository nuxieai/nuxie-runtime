//! Node: `cxx:Function:Luau.VM:VM/src/ldo.cpp:162:lua_d_throw`
//! Source: `VM/src/ldo.cpp` (ldo.cpp:162-165).
//! Rust adaptation: carry the same state/status to the protected boundary as
//! a returned error. Callers must propagate it rather than unwind the host.

use crate::records::lua_exception::{lua_exception, LuaResult};
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn luaD_throw<T>(l: *mut lua_State, errcode: core::ffi::c_int) -> LuaResult<T> {
    Err(lua_exception::lua_exception(l, errcode))
}

#[allow(unused_imports)]
pub use luaD_throw as lua_d_throw;
