#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BuiltinInfo {
    pub params: core::ffi::c_int,
    pub results: core::ffi::c_int,
    pub flags: core::ffi::c_uint,
}

#[allow(non_upper_case_globals)]
impl BuiltinInfo {
    pub const Flag_NoneSafe: core::ffi::c_uint = 1 << 0;
}
