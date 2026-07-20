#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct LocVar {
    pub varname: *mut crate::records::t_string::TString,
    pub startpc: core::ffi::c_int,
    pub endpc: core::ffi::c_int,
    pub reg: u8,
}

#[allow(non_camel_case_types)]
pub type loc_var = LocVar;

impl Default for LocVar {
    fn default() -> Self {
        Self {
            varname: core::ptr::null_mut(),
            startpc: 0,
            endpc: 0,
            reg: 0,
        }
    }
}
