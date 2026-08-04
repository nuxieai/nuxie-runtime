pub type lua_EmbedderMark =
    Option<unsafe extern "C" fn(*mut crate::records::lua_state::lua_State, core::ffi::c_int)>;
