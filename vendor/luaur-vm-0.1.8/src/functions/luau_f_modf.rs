use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luauF_modf(
    _l: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 2 && ttisnumber!(arg0) {
        let a1 = nvalue!(arg0);
        let mut ip: f64 = 0.0;
        let fp = a1.fract(); // C++ modf(a1, &ip) => fractional part is returned, integer part via out param
        ip = a1 - fp;

        setnvalue!(res, ip);
        setnvalue!(res.add(1), fp);
        2
    } else {
        -1
    }
}
