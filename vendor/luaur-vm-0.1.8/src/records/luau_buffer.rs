#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(C)]
pub struct LuauBuffer {
    pub(crate) tt: u8,
    pub(crate) marked: u8,
    pub(crate) memcat: u8,
    pub(crate) len: core::ffi::c_uint,
    pub(crate) _align: [u64; 0],
    pub(crate) data: [core::ffi::c_char; 1],
}

#[allow(non_camel_case_types)]
pub type Buffer = LuauBuffer;
