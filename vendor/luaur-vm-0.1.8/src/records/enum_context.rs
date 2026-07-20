#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct EnumContext {
    pub(crate) L: *mut crate::type_aliases::lua_state::lua_State,
    pub(crate) context: *mut core::ffi::c_void,
    pub(crate) node: Option<
        unsafe extern "C" fn(
            context: *mut core::ffi::c_void,
            ptr: *mut core::ffi::c_void,
            tt: u8,
            memcat: u8,
            size: usize,
            name: *const core::ffi::c_char,
        ),
    >,
    pub(crate) edge: Option<
        unsafe extern "C" fn(
            context: *mut core::ffi::c_void,
            from: *mut core::ffi::c_void,
            to: *mut core::ffi::c_void,
            name: *const core::ffi::c_char,
        ),
    >,
}
