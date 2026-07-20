use crate::type_aliases::stk_id::StkId;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct CallS {
    pub(crate) func: StkId,
    pub(crate) nresults: core::ffi::c_int,
}
