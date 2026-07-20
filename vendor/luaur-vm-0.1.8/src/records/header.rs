#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Header {
    pub(crate) L: *mut crate::records::lua_state::LuaState,
    pub(crate) islittle: core::ffi::c_int,
    pub(crate) maxalign: core::ffi::c_int,
}

#[allow(non_camel_case_types)]
pub type header = Header;

impl Default for Header {
    fn default() -> Self {
        Self {
            L: core::ptr::null_mut(),
            islittle: 0,
            maxalign: 0,
        }
    }
}
