pub type lua_CategoryName = Option<
    unsafe extern "C" fn(*mut crate::records::lua_state::lua_State, u8) -> *const core::ffi::c_char,
>;
