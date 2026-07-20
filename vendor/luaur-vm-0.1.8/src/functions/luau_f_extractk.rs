use crate::enums::lua_type::lua_Type;
use crate::macros::luai_num_2_unsigned::luai_num2unsigned;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_extractk(
    _l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttisnumber!(arg0) {
        let a1 = nvalue!(arg0);
        let a2 = nvalue!(args);

        let mut n: u32 = 0;
        luai_num2unsigned(&mut n, a1);
        let fw = a2 as i32;

        let f = fw & 31;
        let w1 = fw >> 5;

        let m: u32 = !(0xfffffffe_u32 << w1);
        let r = (n >> f) & m;

        setnvalue!(res, r as f64);
        1
    } else {
        -1
    }
}
