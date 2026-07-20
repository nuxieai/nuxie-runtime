use crate::functions::lua_l_addlstring::lua_l_addlstring;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use core::ffi::c_char;
use core::ffi::CStr;

#[inline]
pub fn lua_l_addstring(b: *mut LuaLStrbuf, s: *const c_char) {
    unsafe {
        let len = CStr::from_ptr(s).to_bytes().len();
        lua_l_addlstring(b, s, len);
    }
}
