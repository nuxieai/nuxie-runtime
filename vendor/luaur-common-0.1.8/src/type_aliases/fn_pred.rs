#[allow(non_camel_case_types)]
pub type FnPred = unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> bool;
