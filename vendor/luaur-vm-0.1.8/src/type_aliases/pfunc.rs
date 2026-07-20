#[allow(non_camel_case_types)]
pub type pfunc = Option<
    unsafe fn(L: *mut crate::type_aliases::lua_state::lua_State, ud: *mut core::ffi::c_void),
>;

pub type Pfunc = pfunc;
