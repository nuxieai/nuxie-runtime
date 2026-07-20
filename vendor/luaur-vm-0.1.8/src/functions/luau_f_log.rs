use crate::enums::lua_type::lua_Type;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_log(
    _l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 && ttisnumber!(arg0) {
        let a1 = nvalue!(arg0);

        if nparams == 1 {
            setnvalue!(res, a1.ln());
            1
        } else if ttisnumber!(args) {
            let a2 = nvalue!(args);

            if a2 == 2.0 {
                setnvalue!(res, a1.log2());
                1
            } else if a2 == 10.0 {
                setnvalue!(res, a1.log10());
                1
            } else {
                setnvalue!(res, a1.ln() / a2.ln());
                1
            }
        } else {
            -1
        }
    } else {
        -1
    }
}
