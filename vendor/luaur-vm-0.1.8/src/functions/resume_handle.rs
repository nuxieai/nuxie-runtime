use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_seterrorobj::luaD_seterrorobj;
use crate::functions::lua_f_close::luaF_close;
use crate::functions::luau_poscall::luau_poscall;
use crate::functions::restore_stack_limit::restore_stack_limit;
use crate::functions::resume_continue::resume_continue;
use crate::macros::ci_func::ci_func;
use crate::macros::lua_callinfo_handle::LUA_CALLINFO_HANDLE;
use crate::macros::restoreci::restoreci;
use crate::macros::saveci::saveci;
use crate::records::call_info::CallInfo;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn resume_handle(l: *mut lua_State, ud: *mut core::ffi::c_void) {
    let ci = ud as *mut CallInfo;
    let cl = ci_func!(ci);

    LUAU_ASSERT!(((*ci).flags & LUA_CALLINFO_HANDLE as u32) != 0);
    let c = core::ptr::addr_of!((*cl).inner.c).cast::<crate::records::closure::CClosure>();
    LUAU_ASSERT!((*cl).isC != 0 && (*c).cont.is_some());
    LUAU_ASSERT!((*l).status != 0);

    if !luaur_common::FFlag::LuauResumeRestoreCcalls.get() {
        (*l).nCcalls = (*l).baseCcalls;
    }

    (*ci).flags &= !(LUA_CALLINFO_HANDLE as u32);

    let status = (*l).status as i32;
    (*l).status = lua_Status::LUA_OK as u8;

    if status != lua_Status::LUA_ERRRUN as i32 {
        luaD_seterrorobj(l, status, (*l).top);
    }

    (*l).base = (*ci).base;
    (*ci).top = (*l).top;

    let old_ci = saveci!(l, ci);

    let n = (*c).cont.unwrap()(l, status);

    (*l).ci = restoreci!(l, old_ci);

    luaF_close(l, (*(*l).ci).base);
    restore_stack_limit(l);

    luau_poscall(l, (*l).top.offset(-(n as isize)));
    resume_continue(l);
}
