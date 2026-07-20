//! Node: `cxx:Function:Luau.VM:VM/src/lvmexecute.cpp:3843:luau_poscall`
//! Source: `VM/src/lvmexecute.cpp:3843-3872` (hand-ported)

use crate::macros::clvalue::clvalue;
use crate::macros::lua_multret::LUA_MULTRET;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::setobj_2_s::setobj_2_s;
use crate::records::closure::Closure;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

/// C++ `void luau_poscall(lua_State* L, StkId first)`.
#[allow(non_snake_case)]
pub unsafe fn luau_poscall(L: *mut lua_State, first: StkId) {
    // finish interrupted execution of `OP_CALL'
    // ci is our callinfo, cip is our parent
    let ci = (*L).ci;
    let cip = ci.sub(1);

    if luaur_common::FFlag::LuauClosureUsageCounter.get() {
        let cicl = clvalue!((*ci).func);
        LUAU_ASSERT!((*cicl).usage > 0);
        (*cicl).usage -= 1;
    }

    // copy return values into parent stack (but only up to nresults!), fill
    // the rest with nil
    // TODO: it might be worthwhile to handle the case when nresults==b explicitly?
    let mut res = (*ci).func;
    let mut vali = first;
    let valend = (*L).top;

    let mut i = (*ci).nresults;
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
    (*L).top = if (*ci).nresults == LUA_MULTRET {
        res
    } else {
        (*cip).top
    };
}
