use core::ffi::{c_int, c_void};

use crate::functions::lua_d_callint::lua_d_callint;
use crate::functions::lua_d_pcall::luaD_pcall;
use crate::functions::lua_isyieldable::lua_isyieldable;
use crate::macros::api_check::api_check;
use crate::macros::c_call_yield::C_CALL_YIELD;
use crate::macros::clvalue::clvalue;
use crate::macros::expandstacklimit::expandstacklimit;
use crate::macros::iscfunction::iscfunction;
use crate::macros::isyielded::isyielded;
use crate::macros::lua_callinfo_handle::LUA_CALLINFO_HANDLE;
use crate::macros::savestack::savestack;
use crate::records::closure::CClosure;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

struct CallContext {
    func: StkId,
    nresults: c_int,
}

unsafe fn call_context_run(L: *mut lua_State, ud: *mut c_void) {
    let ctx = ud as *mut CallContext;
    lua_d_callint(L, (*ctx).func, (*ctx).nresults, lua_isyieldable(L) != 0);
}

#[allow(non_snake_case)]
pub unsafe fn lua_l_pcallyieldable(
    L: *mut lua_State,
    nargs: c_int,
    nresults: c_int,
    errfunc: c_int,
) -> c_int {
    api_check!(L, iscfunction!((*(*L).ci).func));
    let cl = clvalue!((*(*L).ci).func);
    let c = core::ptr::addr_of!((*cl).inner.c).cast::<CClosure>();
    api_check!(L, (*c).cont.is_some());
    api_check!(L, nargs + 1 <= (*L).top.offset_from((*L).base) as c_int);
    api_check!(
        L,
        errfunc >= 0 && errfunc <= (*L).top.offset_from((*L).base) as c_int
    );

    (*(*L).ci).context.errfunc = errfunc;
    (*(*L).ci).flags |= LUA_CALLINFO_HANDLE as u32;

    let mut ctx = CallContext {
        func: (*L).top.sub((nargs + 1) as usize),
        nresults,
    };

    let savedfunc = savestack!(L, ctx.func) as isize;
    let savederrfunc = if errfunc != 0 {
        savestack!(L, (*L).base.add((errfunc - 1) as usize)) as isize
    } else {
        0
    };

    let status = luaD_pcall(
        L,
        Some(call_context_run),
        &mut ctx as *mut CallContext as *mut c_void,
        savedfunc,
        savederrfunc,
    );

    expandstacklimit!(L, (*L).top);

    if status == 0 && isyielded(L) {
        return C_CALL_YIELD;
    }

    (*(*L).ci).flags &= !(LUA_CALLINFO_HANDLE as u32);

    (*c).cont.unwrap()(L, status)
}
