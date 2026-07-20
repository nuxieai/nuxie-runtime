use crate::enums::lua_status::lua_Status;
use crate::functions::lua_c_barrierback::lua_c_barrierback;
use crate::functions::resume_error::resume_error;
use crate::macros::api_check::api_check;
use crate::macros::isblack::isblack;
use crate::macros::luai_maxccalls::LUAI_MAXCCALLS;
use crate::records::gc_object::GCObject;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn resume_start(l: *mut lua_State, from: *mut lua_State, nargs: c_int) -> c_int {
    api_check!(l, nargs >= 0);
    api_check!(l, (*l).top.offset_from((*l).base) >= nargs as isize);

    if (*l).status != lua_Status::LUA_YIELD as u8
        && (*l).status != lua_Status::LUA_BREAK as u8
        && ((*l).status != 0 || (*l).ci != (*l).base_ci)
    {
        return resume_error(l, c"cannot resume non-suspended coroutine".as_ptr(), nargs);
    }

    (*l).nCcalls = if !from.is_null() { (*from).nCcalls } else { 0 };
    if (*l).nCcalls as i32 >= LUAI_MAXCCALLS {
        return resume_error(l, c"C stack overflow".as_ptr(), nargs);
    }

    (*l).nCcalls = (*l).nCcalls.wrapping_add(1);
    (*l).baseCcalls = (*l).nCcalls;
    (*l).isactive = true;

    let o = l as *mut GCObject;
    if isblack!(o) {
        lua_c_barrierback(l, o, core::ptr::addr_of_mut!((*l).gclist));
    }

    lua_Status::LUA_OK as c_int
}
