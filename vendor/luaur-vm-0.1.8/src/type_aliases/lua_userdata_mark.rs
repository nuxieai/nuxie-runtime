pub type lua_UserdataMark =
    Option<unsafe extern "C" fn(*mut crate::records::lua_state::lua_State, *mut core::ffi::c_void)>;
