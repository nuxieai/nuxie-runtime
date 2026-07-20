//! Node: `cxx:Function:Luau.VM:VM/src/lvmexecute.cpp:3757:luau_precall`
//! Source: `VM/src/lvmexecute.cpp:3757-3841` (hand-ported)

use crate::functions::lua_v_tryfunc_tm::lua_v_tryfunc_tm;
use crate::macros::clvalue::clvalue;
use crate::macros::incr_ci::incr_ci;
use crate::macros::lua_callinfo_native::LUA_CALLINFO_NATIVE;
use crate::macros::lua_d_checkstackfornewci::luaD_checkstackfornewci;
use crate::macros::pcrc::PCRC;
use crate::macros::pcrlua::PCRLUA;
use crate::macros::pcryield::PCRYIELD;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::setobj_2_s::setobj_2_s;
use crate::macros::ttisfunction::ttisfunction;
use crate::records::closure::Closure;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

/// C++ `int luau_precall(lua_State* L, StkId func, int nresults)`.
#[allow(non_snake_case)]
pub unsafe fn luau_precall(
    L: *mut lua_State,
    func: StkId,
    nresults: core::ffi::c_int,
) -> core::ffi::c_int {
    if !ttisfunction!(func as *const TValue) {
        lua_v_tryfunc_tm(L, func);
        // L->top is incremented by tryfuncTM
    }

    let ccl = clvalue!(func as *const TValue);

    incr_ci!(L);
    let ci = (*L).ci;
    (*ci).func = func;
    (*ci).base = func.add(1);
    (*ci).top = (*L).top.add((*ccl).stacksize as usize);
    (*ci).savedpc = core::ptr::null();
    (*ci).flags = 0;
    (*ci).nresults = nresults;
    if luaur_common::FFlag::LuauClosureUsageCounter.get() {
        (*ccl).usage += 1;
    }

    (*L).base = (*ci).base;
    // Note: L->top is assigned externally

    luaD_checkstackfornewci(L, (*ccl).stacksize as i32);
    LUAU_ASSERT!((*ci).top <= (*L).stack_last);

    if (*ccl).isC == 0 {
        let p = {
            let l = &(*ccl).inner.l;
            l.p
        };

        // fill unused parameters with nil
        let mut argi = (*L).top;
        let argend = (*L).base.add((*p).numparams as usize);
        while argi < argend {
            setnilvalue!(argi); // complete missing arguments
            argi = argi.add(1);
        }
        (*L).top = if (*p).is_vararg != 0 { argi } else { (*ci).top };

        (*ci).savedpc = (*p).code;

        // VM_HAS_NATIVE
        if (*p).exectarget != 0 && !(*p).execdata.is_null() {
            (*ci).flags = LUA_CALLINFO_NATIVE as u32;
        }

        PCRLUA
    } else {
        let f = {
            let c = &(*ccl).inner.c;
            c.f
        };
        let n = match f {
            Some(f) => f(L),
            None => 0,
        };

        // yield
        if n < 0 {
            return PCRYIELD;
        }

        // ci is our callinfo, cip is our parent
        let ci = (*L).ci;
        let cip = ci.sub(1);

        if luaur_common::FFlag::LuauClosureUsageCounter.get() {
            LUAU_ASSERT!((*ccl).usage > 0);
            (*ccl).usage -= 1;
        }

        // copy return values into parent stack (but only up to nresults!),
        // fill the rest with nil
        // TODO: it might be worthwhile to handle the case when nresults==b explicitly?
        let mut res = (*ci).func;
        let mut vali = (*L).top.sub(n as usize);
        let valend = (*L).top;

        let mut i = nresults;
        while i != 0 && vali < valend {
            setobj_2_s!(L, res, vali as *const TValue);
            res = res.add(1);
            vali = vali.add(1);
            i -= 1;
        }
        while i > 0 {
            setnilvalue!(res);
            res = res.add(1);
            i -= 1;
        }

        // pop the stack frame
        (*L).ci = cip;
        (*L).base = (*cip).base;
        (*L).top = res;

        PCRC
    }
}
