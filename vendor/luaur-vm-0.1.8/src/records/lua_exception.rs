use crate::functions::lua_tolstring::lua_tolstring;
use crate::records::lua_state::lua_State;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct lua_exception {
    pub(crate) L: *mut lua_State,
    pub(crate) status: core::ffi::c_int,
}

impl lua_exception {
    #[allow(non_snake_case)]
    pub fn lua_exception(L: *mut lua_State, status: core::ffi::c_int) -> Self {
        Self { L, status }
    }

    #[allow(non_snake_case)]
    pub fn what(&self) -> *const core::ffi::c_char {
        // LUA_ERRRUN passes error object on the stack
        if self.status == 2 {
            // LUA_ERRRUN is 2
            unsafe {
                let val = lua_tolstring(self.L, -1, core::ptr::null_mut());
                if !val.is_null() {
                    return val;
                }
            }
        }

        match self.status {
            2 => c"lua_exception: runtime error".as_ptr(), // LUA_ERRRUN
            3 => c"lua_exception: syntax error".as_ptr(),  // LUA_ERRSYNTAX
            4 => c"lua_exception: memory allocation error: block too big".as_ptr(), // LUA_ERRMEM + LUA_MEMERRMSG
            5 => c"lua_exception: error in error handling".as_ptr(), // LUA_ERRERR + LUA_ERRERRMSG
            _ => c"lua_exception: unexpected exception status".as_ptr(),
        }
    }

    #[allow(non_snake_case)]
    pub fn getStatus(&self) -> core::ffi::c_int {
        self.status
    }

    #[allow(non_snake_case)]
    pub fn getThread(&self) -> *const lua_State {
        self.L as *const lua_State
    }
}

// The exception unwinds within one thread (C++ throw/catch semantics);
// panic_any requires Send, which the raw lua_State pointer doesn't derive.
unsafe impl Send for lua_exception {}
