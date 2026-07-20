#[allow(non_camel_case_types)]
pub type lua_CFunction =
    Option<unsafe fn(L: *mut crate::type_aliases::lua_state::lua_State) -> core::ffi::c_int>;

pub type LuaCFunction = lua_CFunction;
