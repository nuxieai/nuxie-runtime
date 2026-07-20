use crate::enums::lua_type::lua_Type;
use crate::macros::lvalue::lvalue;
use crate::macros::setlvalue::setlvalue;
use crate::macros::ttisinteger::ttisinteger;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_integermin(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttisinteger!(arg0) && ttisinteger!(args) {
        let a1: i64 = lvalue!(arg0);
        let a2: i64 = lvalue!(args);

        let mut r = if a2 < a1 { a2 } else { a1 };

        for i in 3..=nparams {
            let idx = (i - 2) as isize;
            if !ttisinteger!(args.offset(idx)) {
                return -1;
            }

            let a: i64 = lvalue!(args.offset(idx));
            r = if a < r { a } else { r };
        }

        setlvalue!(res, r);
        1
    } else {
        -1
    }
}
