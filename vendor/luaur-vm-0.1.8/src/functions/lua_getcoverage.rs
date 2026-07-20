use crate::functions::getcoverage::getcoverage;
use crate::functions::getmaxline::getmaxline;
use crate::functions::lua_a_toobject::luaA_toobject;
use crate::macros::api_check::api_check;
use crate::macros::clvalue::clvalue;
use crate::macros::lua_m_freearray::luaM_freearray;
use crate::macros::lua_m_newarray::luaM_newarray;
use crate::macros::ttisfunction::ttisfunction;
use crate::records::proto::Proto;
use crate::type_aliases::lua_coverage::lua_Coverage;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_getcoverage(
    L: *mut lua_State,
    funcindex: core::ffi::c_int,
    context: *mut core::ffi::c_void,
    callback: lua_Coverage,
) {
    let func: *const TValue = luaA_toobject(L, funcindex);
    api_check!(L, ttisfunction!(func) && (*clvalue!(func)).isC == 0);

    let cl = clvalue!(func);
    let lcl = core::ptr::addr_of!((*cl).inner.l).cast::<crate::records::closure::LClosure>();
    let p = (*lcl).p as *mut Proto;

    let size = getmaxline(p) as usize + 1;
    if size == 0 {
        return;
    }

    let buffer = luaM_newarray!(L, size, core::ffi::c_int, 0);

    getcoverage(p, 0, buffer, size, context, callback);

    luaM_freearray!(
        L,
        buffer as *mut core::ffi::c_void,
        size,
        core::ffi::c_int,
        0
    );
}
