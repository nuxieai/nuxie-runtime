use crate::functions::lua_d_rawrunprotected_ldo::luaD_rawrunprotected;
use crate::functions::resume::resume;
use crate::functions::resume_finish::resume_finish;
use crate::functions::resume_start::resume_start;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn lua_resume(l: *mut lua_State, from: *mut lua_State, nargs: c_int) -> c_int {
    let starterror = resume_start(l, from, nargs);
    if starterror != 0 {
        return starterror;
    }

    let oldnCcalls = (*l).nCcalls as c_int;
    let status = luaD_rawrunprotected(
        l,
        Some(resume),
        (*l).top.offset(-(nargs as isize)) as *mut core::ffi::c_void,
    );

    resume_finish(l, status, oldnCcalls)
}
