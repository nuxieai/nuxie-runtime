pub type lua_EmbedderGc = Option<
    unsafe extern "C" fn(
        *mut crate::records::lua_state::lua_State,
        crate::type_aliases::lua_embedder_mark::lua_EmbedderMark,
    ),
>;
