use crate::macros::clvalue::clvalue;
use crate::macros::incr_ci::incr_ci;
use crate::macros::lua_d_checkstack::luaD_checkstack;
use crate::macros::lua_minstack::LUA_MINSTACK;
use crate::macros::luai_maxccalls::LUAI_MAXCCALLS;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::setobj_2_s::setobj2s;
use crate::macros::ttisfunction::ttisfunction;
use crate::records::call_info::CallInfo;
use crate::records::closure::Closure;
use crate::records::lua_state::lua_State;
use crate::type_aliases::lua_c_function::lua_CFunction;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::FFlag;

/// C++ `LUAU_NOINLINE void luaV_callTM(lua_State* L, int nparams, int res)`.
#[allow(non_snake_case)]
pub unsafe fn lua_v_call_tm(L: *mut lua_State, nparams: i32, res: i32) {
    // LUAU_NOINLINE is handled by the attribute on the function
    (*L).nCcalls += 1;

    if (*L).nCcalls >= LUAI_MAXCCALLS as u16 {
        crate::functions::lua_d_check_cstack::luaD_checkCstack(L);
    }

    luaD_checkstack!(L, LUA_MINSTACK);

    let top = (*L).top;
    let fun = top.sub(nparams as usize).sub(1);

    let ci = incr_ci!(L);
    (*ci).func = fun;
    (*ci).base = fun.add(1);
    (*ci).top = top.add(LUA_MINSTACK as usize);
    (*ci).savedpc = std::ptr::null_mut();
    (*ci).flags = 0;
    (*ci).nresults = if res >= 0 { 1 } else { 0 };
    LUAU_ASSERT!((*ci).top <= (*L).stack_last);

    let mut ccl: *mut Closure = std::ptr::null_mut();
    if FFlag::LuauClosureUsageCounter.get() {
        ccl = clvalue!(fun) as *mut Closure;
        (*ccl).usage += 1;
    }

    LUAU_ASSERT!(ttisfunction!((*ci).func));
    LUAU_ASSERT!((clvalue!((*ci).func) as *mut Closure).is_null() == false);
    LUAU_ASSERT!((clvalue!((*ci).func) as *mut Closure).is_null() == false);
    LUAU_ASSERT!((*clvalue!((*ci).func)).isC != 0);

    (*L).base = fun.add(1);
    LUAU_ASSERT!((*L).top == (*L).base.add(nparams as usize));

    let cl = clvalue!(fun);
    let c = core::ptr::addr_of!((*cl).inner.c).cast::<crate::records::closure::CClosure>();
    let func = (*c).f;
    let n = func.unwrap()(L);
    LUAU_ASSERT!(n >= 0); // yields should have been blocked by nCcalls

    // ci is our callinfo, cip is our parent
    // note that we read L->ci again since it may have been reallocated by the call
    let cip = (*L).ci.sub(1);

    if FFlag::LuauClosureUsageCounter.get() {
        LUAU_ASSERT!((*ccl).usage > 0);
        (*ccl).usage -= 1;
    }

    // copy return value into parent stack
    if res >= 0 {
        if n > 0 {
            setobj2s!(
                L,
                (*cip).base.add(res as usize),
                (*L).top.sub(n as usize) as *const TValue
            );
        } else {
            setnilvalue!((*cip).base.add(res as usize));
        }
    }

    (*L).ci = cip;
    (*L).base = (*cip).base;
    (*L).top = (*cip).top;

    (*L).nCcalls -= 1;
}
