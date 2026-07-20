use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn luau_f_ldexp(
    _l: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: c_int,
    args: StkId,
    nparams: c_int,
) -> c_int {
    if nparams >= 2 && nresults <= 1 && ttisnumber!(arg0) && ttisnumber!(args) {
        let a1 = nvalue!(arg0);
        let a2 = nvalue!(args);
        setnvalue!(res, (a1 * 2f64.powi(a2 as i32)) as f64);
        1
    } else {
        -1
    }
}
