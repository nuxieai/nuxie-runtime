use crate::records::proto::Proto;
use crate::type_aliases::stk_id::StkId;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
#[repr(C)]
pub union CallInfoContext {
    pub savedpc: *const u32,
    pub errfunc: core::ffi::c_int,
}

impl core::fmt::Debug for CallInfoContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CallInfoContext").finish_non_exhaustive()
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CallInfo {
    pub base: StkId,
    pub func: StkId,
    pub top: StkId,
    pub p: *mut Proto,
    pub context: CallInfoContext,
    pub nresults: core::ffi::c_int,
    pub flags: core::ffi::c_uint,
}

#[allow(non_camel_case_types)]
pub type call_info = CallInfo;

impl Default for CallInfo {
    fn default() -> Self {
        Self {
            base: core::ptr::null_mut(),
            func: core::ptr::null_mut(),
            top: core::ptr::null_mut(),
            p: core::ptr::null_mut(),
            context: CallInfoContext {
                savedpc: core::ptr::null(),
            },
            nresults: 0,
            flags: 0,
        }
    }
}
