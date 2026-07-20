#[allow(non_camel_case_types)]
pub type lua_Hook = Option<
    unsafe extern "C" fn(
        L: *mut crate::type_aliases::lua_state::lua_State,
        ar: *mut crate::records::lua_debug::LuaDebug,
    ),
>;

pub type LuaHook = lua_Hook;
