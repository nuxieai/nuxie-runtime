#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct stringtable {
    pub(crate) hash: *mut *mut crate::records::t_string::TString,
    pub(crate) nuse: u32,
    pub(crate) size: core::ffi::c_int,
}

#[allow(non_camel_case_types)]
pub type Stringtable = stringtable;

impl Default for stringtable {
    fn default() -> Self {
        Self {
            hash: core::ptr::null_mut(),
            nuse: 0,
            size: 0,
        }
    }
}
