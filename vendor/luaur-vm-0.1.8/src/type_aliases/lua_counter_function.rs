#[allow(non_camel_case_types)]
pub type lua_CounterFunction = Option<
    unsafe extern "C" fn(
        context: *mut core::ffi::c_void,
        function: *const core::ffi::c_char,
        linedefined: core::ffi::c_int,
    ),
>;

#[allow(non_camel_case_types)]
pub type LuaCounterFunction = lua_CounterFunction;
