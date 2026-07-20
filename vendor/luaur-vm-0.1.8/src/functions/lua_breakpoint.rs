use crate::functions::getnextline::getnextline;
use crate::functions::lua_a_toobject::luaA_toobject;
use crate::functions::lua_g_breakpoint::lua_g_breakpoint;
use crate::macros::api_check::api_check;
use crate::macros::clvalue::clvalue;
use crate::macros::ttisfunction::ttisfunction;
use crate::records::proto::Proto;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::proto::Proto as ProtoType;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_breakpoint(
    L: *mut lua_State,
    funcindex: core::ffi::c_int,
    line: core::ffi::c_int,
    enabled: core::ffi::c_int,
) -> core::ffi::c_int {
    let func: *const TValue = luaA_toobject(L, funcindex);
    api_check!(L, ttisfunction!(func) && (*clvalue!(func)).isC == 0);

    let cl = clvalue!(func);
    let lcl = core::ptr::addr_of!((*cl).inner.l).cast::<crate::records::closure::LClosure>();
    let p: *mut Proto = (*lcl).p;

    let target = getnextline(p, line);

    if target != -1 {
        lua_g_breakpoint(L, p, target, enabled != 0);
    }

    target
}
