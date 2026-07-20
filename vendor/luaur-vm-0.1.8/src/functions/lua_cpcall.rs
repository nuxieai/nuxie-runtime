use crate::functions::f_ccall::f_ccall;
use crate::functions::lua_d_pcall::lua_d_pcall;
use crate::macros::api_check::api_check;
use crate::macros::savestack::savestack;
use crate::records::c_call_s::CCallS;
use crate::type_aliases::lua_c_function::lua_CFunction;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_cpcall(
    L: *mut lua_State,
    func: lua_CFunction,
    ud: *mut core::ffi::c_void,
) -> core::ffi::c_int {
    api_check!(L, (*L).status == 0);
    api_check!(L, func.is_some());

    let mut c = CCallS { func, ud };

    lua_d_pcall(
        L,
        Some(f_ccall),
        &mut c as *mut CCallS as *mut core::ffi::c_void,
        savestack!(L, (*L).top) as isize,
        0,
    )
}
