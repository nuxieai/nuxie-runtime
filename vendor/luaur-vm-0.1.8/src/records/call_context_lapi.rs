#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct CallContext {
    pub(crate) size: core::ffi::c_int,
}
