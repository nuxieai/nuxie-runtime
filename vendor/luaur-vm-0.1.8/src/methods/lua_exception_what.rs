use crate::enums::lua_status::lua_Status;
use crate::macros::lua_errerrmsg::LUA_ERRERRMSG;
use crate::macros::lua_memerrmsg::LUA_MEMERRMSG;
use crate::macros::lua_tostring::lua_tostring;
use crate::records::lua_exception::lua_exception;

#[cfg(any())]
impl lua_exception {
    /// C++ `const char* what() const throw() override`
    pub fn what(&self) -> *const core::ffi::c_char {
        // LUA_ERRRUN passes error object on the stack
        if self.status == lua_Status::LUA_ERRRUN as core::ffi::c_int {
            // SAFETY: lua_tostring is expected to handle invalid stack indices appropriately.
            if let Some(str) = unsafe { lua_tostring!(self.L, -1) }.as_ref() {
                return str.as_ptr();
            }
        }

        match self.status {
            x if x == lua_Status::LUA_ERRRUN as core::ffi::c_int => {
                b"lua_exception: runtime error\0".as_ptr() as *const core::ffi::c_char
            }
            x if x == lua_Status::LUA_ERRSYNTAX as core::ffi::c_int => {
                b"lua_exception: syntax error\0".as_ptr() as *const core::ffi::c_char
            }
            x if x == lua_Status::LUA_ERRMEM as core::ffi::c_int => LUA_MEMERRMSG,
            x if x == lua_Status::LUA_ERRERR as core::ffi::c_int => LUA_ERRERRMSG,
            _ => {
                b"lua_exception: unexpected exception status\0".as_ptr() as *const core::ffi::c_char
            }
        }
    }
}
