use crate::records::t_string::TString;

#[allow(non_snake_case)]
#[inline]
pub unsafe fn getstr(ts: *const TString) -> *const core::ffi::c_char {
    (*ts).data.as_ptr()
}

#[allow(non_snake_case)]
#[inline]
pub unsafe fn getstr_mut(ts: *mut TString) -> *mut core::ffi::c_char {
    (*ts).data.as_mut_ptr()
}
