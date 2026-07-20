use crate::functions::lua_d_call::lua_d_call;
use crate::macros::cast_to::cast_to;
use crate::records::call_s::CallS;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn f_call(L: *mut lua_State, ud: *mut core::ffi::c_void) {
    let c = cast_to!(*mut CallS, ud);
    lua_d_call(L, (*c).func, (*c).nresults);
}
