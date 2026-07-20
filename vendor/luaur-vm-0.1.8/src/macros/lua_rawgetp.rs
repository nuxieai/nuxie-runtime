use crate::functions::lua_rawgetptagged::lua_rawgetptagged;

#[allow(non_snake_case)]
pub unsafe fn lua_rawgetp(
    l: *mut crate::records::lua_state::lua_State,
    idx: core::ffi::c_int,
    p: *mut core::ffi::c_void,
) -> core::ffi::c_int {
    #[cfg(not(feature = "internal_stub_resolution"))]
    {
        lua_rawgetptagged(l, idx, p, 0)
    }

    #[cfg(feature = "internal_stub_resolution")]
    {
        0
    }
}
