use crate::functions::lua_h_getnum::lua_h_getnum;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_c_threadbarrier::luaC_threadbarrier;
use crate::macros::setobj_2_s::setobj2s;
use crate::records::lua_state::lua_State;

pub unsafe fn lua_getweakref(l: *mut lua_State, reference: core::ffi::c_int) -> core::ffi::c_int {
    luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauGcTraceUdata.get());
    luaC_threadbarrier!(l);
    crate::ensure_stack!(l, 1);
    let wr = hvalue!(core::ptr::addr_of!((*(*l).global).weakregistry));
    setobj2s!(l, (*l).top, lua_h_getnum(wr, reference));
    api_incr_top!(l);
    (*(*l).top.sub(1)).tt
}
