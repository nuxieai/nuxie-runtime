use crate::enums::lua_type::lua_Type;
use crate::macros::lvalue::lvalue;
use crate::macros::setlvalue::setlvalue;
use crate::macros::ttisinteger::ttisinteger;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_integeridiv(
    _l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttisinteger!(arg0) && ttisinteger!(args) {
        let a1: i64 = lvalue!(arg0);
        let a2: i64 = lvalue!(args);

        if a2 == 0 {
            return -1;
        }
        if a1 == i64::MIN && a2 == -1 {
            return -1;
        }

        let result = a1 / a2;
        if result < 0 && (a1 % a2) != 0 {
            setlvalue!(res, result - 1);
        } else {
            setlvalue!(res, result);
        }
        1
    } else {
        -1
    }
}
