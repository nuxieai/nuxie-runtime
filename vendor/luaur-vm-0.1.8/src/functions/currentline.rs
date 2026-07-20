use crate::functions::currentpc::currentpc;
use crate::functions::lua_g_getline::luaG_getline;
use crate::macros::ci_func::ci_func;
use crate::type_aliases::call_info::CallInfo;
use crate::type_aliases::lua_state::lua_State;

pub(crate) unsafe fn currentline(_l: *mut lua_State, ci: *mut CallInfo) -> core::ffi::c_int {
    let cl = ci_func!(ci);
    let lcl = core::ptr::addr_of!((*cl).inner.l).cast::<crate::records::closure::LClosure>();
    luaG_getline((*lcl).p, currentpc(_l, ci))
}
