#[allow(non_camel_case_types)]
pub type lua_Coverage = Option<
    unsafe extern "C" fn(
        context: *mut core::ffi::c_void,
        function: *const core::ffi::c_char,
        linedefined: core::ffi::c_int,
        depth: core::ffi::c_int,
        hits: *const core::ffi::c_int,
        size: usize,
    ),
>;

#[allow(non_camel_case_types)]
pub type LuaCoverage = lua_Coverage;
