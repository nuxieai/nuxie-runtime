#[allow(non_camel_case_types)]
pub type lua_CounterValue = Option<
    unsafe extern "C" fn(
        context: *mut core::ffi::c_void,
        kind: core::ffi::c_int,
        line: core::ffi::c_int,
        hits: u64,
    ),
>;

#[allow(non_camel_case_types)]
pub type LuaCounterValue = lua_CounterValue;
