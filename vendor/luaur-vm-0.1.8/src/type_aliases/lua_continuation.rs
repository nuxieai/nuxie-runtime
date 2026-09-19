#[allow(non_camel_case_types)]
pub type lua_Continuation = Option<
    unsafe fn(
        L: *mut crate::type_aliases::lua_state::lua_State,
        status: core::ffi::c_int,
    ) -> crate::records::lua_exception::LuaResult<core::ffi::c_int>,
>;

pub type LuaContinuation = lua_Continuation;
