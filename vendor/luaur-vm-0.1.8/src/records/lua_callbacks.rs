//! Node: `cxx:Record:Luau.VM:VM/include/lua.h:511:lua_callbacks`
//! Source: `VM/include/lua.h` (lua.h:511-527, hand-ported)

use crate::records::lua_debug::LuaDebug as lua_Debug;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int, c_void};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct LuaCallbacks {
    /// arbitrary userdata pointer that is never overwritten by Luau
    pub userdata: *mut c_void,

    /// gets called at safepoints (loop back edges, call/ret, gc) if set
    pub interrupt: Option<unsafe extern "C-unwind" fn(l: *mut lua_State, gc: c_int)>,
    /// gets called when an unprotected error is raised (if longjmp is used)
    pub panic: Option<unsafe extern "C" fn(l: *mut lua_State, errcode: c_int)>,

    /// gets called when L is created (LP == parent) or destroyed (LP == NULL)
    pub userthread: Option<unsafe extern "C" fn(lp: *mut lua_State, l: *mut lua_State)>,
    /// gets called when a string is created to assign an atom id
    pub useratom:
        Option<unsafe extern "C" fn(l: *mut lua_State, s: *const c_char, len: usize) -> i16>,

    /// gets called when BREAK instruction is encountered
    pub debugbreak: Option<unsafe extern "C" fn(l: *mut lua_State, ar: *mut lua_Debug)>,
    /// gets called after each instruction in single step mode
    pub debugstep: Option<unsafe extern "C" fn(l: *mut lua_State, ar: *mut lua_Debug)>,
    /// gets called when thread execution is interrupted by break in another thread
    pub debuginterrupt: Option<unsafe extern "C" fn(l: *mut lua_State, ar: *mut lua_Debug)>,
    /// gets called when protected call results in an error
    pub debugprotectederror: Option<unsafe extern "C-unwind" fn(l: *mut lua_State)>,

    /// gets called when memory is allocated
    pub onallocate: Option<unsafe extern "C" fn(l: *mut lua_State, osize: usize, nsize: usize)>,
}

#[allow(non_camel_case_types)]
pub type lua_Callbacks = LuaCallbacks;
