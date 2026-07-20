#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct Udata {
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) memcat: u8,

    pub tag: u8,

    pub len: core::ffi::c_int,

    pub metatable: *mut crate::records::lua_table::LuaTable,

    pub(crate) _align: [u64; 0],
    pub data: [core::ffi::c_char; 1],
}

#[allow(non_camel_case_types)]
pub type udata = Udata;
