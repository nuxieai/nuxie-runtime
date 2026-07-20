use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn luau_f_frexp(
    _L: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: c_int,
    _args: StkId,
    nparams: c_int,
) -> c_int {
    if nparams >= 1 && nresults <= 2 && ttisnumber!(arg0) {
        let a1 = nvalue!(arg0);
        let (f, e) = frexp(a1);
        setnvalue!(res, f);
        setnvalue!(res.add(1), e as f64);
        2
    } else {
        -1
    }
}

fn frexp(value: f64) -> (f64, c_int) {
    if value == 0.0 || !value.is_finite() {
        return (value, 0);
    }

    let e = value.abs().log2().floor() as c_int + 1;
    (value / 2f64.powi(e), e)
}
