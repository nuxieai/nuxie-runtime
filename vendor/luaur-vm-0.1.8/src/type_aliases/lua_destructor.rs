#[allow(non_camel_case_types)]
pub type lua_Destructor = Option<
    unsafe extern "C" fn(
        L: *mut crate::type_aliases::lua_state::lua_State,
        userdata: *mut core::ffi::c_void,
    ),
>;

pub type LuaDestructor = lua_Destructor;
