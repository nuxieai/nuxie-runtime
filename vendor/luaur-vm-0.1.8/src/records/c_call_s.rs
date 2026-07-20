use crate::type_aliases::lua_c_function::lua_CFunction;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct CCallS {
    pub(crate) func: lua_CFunction,
    pub(crate) ud: *mut core::ffi::c_void,
}
