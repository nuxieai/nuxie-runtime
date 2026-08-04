#[allow(non_camel_case_types)]
pub type lua_LibraryMemberTypeCallback = Option<
    unsafe extern "C" fn(
        library: *const core::ffi::c_char,
        member: *const core::ffi::c_char,
    ) -> core::ffi::c_int,
>;

pub type LuaLibraryMemberTypeCallback = lua_LibraryMemberTypeCallback;
