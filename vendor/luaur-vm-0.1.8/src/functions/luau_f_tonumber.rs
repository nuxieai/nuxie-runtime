use crate::functions::lua_o_str_2_d::lua_o_str_2_d;
use crate::macros::nvalue::nvalue;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::svalue::svalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::macros::ttisstring::ttisstring;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_tonumber(
    _l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams == 1 && nresults <= 1 {
        let mut num: f64 = 0.0;

        if ttisnumber!(arg0) {
            setnvalue!(res, nvalue!(arg0));
            1
        } else if ttisstring!(arg0) && lua_o_str_2_d(svalue!(arg0), &mut num) != 0 {
            setnvalue!(res, num);
            1
        } else {
            setnilvalue!(res);
            1
        }
    } else {
        -1
    }
}
