#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(C)]
pub struct TString {
    pub(crate) hdr: crate::records::g_cheader::GCheader,
    pub(crate) _padding1: [core::ffi::c_char; 1],
    pub(crate) atom: i16,
    pub(crate) _padding2: [core::ffi::c_char; 2],
    pub(crate) next: *mut TString,
    pub(crate) hash: core::ffi::c_uint,
    pub(crate) len: core::ffi::c_uint,
    pub(crate) data: [core::ffi::c_char; 1],
}

#[allow(non_camel_case_types)]
pub type t_string = TString;
