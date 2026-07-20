#[allow(non_camel_case_types)]
pub type lua_UserdataDirectAccess = Option<
    unsafe extern "C" fn(
        L: *mut crate::type_aliases::lua_state::lua_State,
        data: *mut core::ffi::c_void,
        atom: core::ffi::c_int,
        cachedslot: *mut u16,
        utag: core::ffi::c_int,
    ),
>;

pub type LuaUserdataDirectAccess = lua_UserdataDirectAccess;
