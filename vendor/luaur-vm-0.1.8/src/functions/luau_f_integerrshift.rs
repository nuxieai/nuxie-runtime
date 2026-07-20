use crate::enums::lua_type::lua_Type;
use crate::macros::lvalue::lvalue;
use crate::macros::setlvalue::setlvalue;
use crate::macros::ttisinteger::ttisinteger;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_integerrshift(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttisinteger!(arg0) && ttisinteger!(args) {
        let n: u64 = lvalue!(arg0) as u64;
        let i: i64 = lvalue!(args);

        setlvalue!(
            res,
            if (i >= -63) && (i <= 63) {
                (if i < 0 { n << (-i) } else { n >> i }) as i64
            } else {
                0
            }
        );

        1
    } else {
        -1
    }
}
