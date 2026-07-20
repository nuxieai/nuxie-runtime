#[allow(non_camel_case_types)]
pub type lua_UserdataDirectNamecall = Option<
    unsafe extern "C" fn(
        L: *mut crate::type_aliases::lua_state::lua_State,
        data: *mut core::ffi::c_void,
        atom: core::ffi::c_int,
        cachedslot: *mut u16,
        utag: core::ffi::c_int,
    ) -> core::ffi::c_int,
>;

pub type LuaUserdataDirectNamecall = lua_UserdataDirectNamecall;
