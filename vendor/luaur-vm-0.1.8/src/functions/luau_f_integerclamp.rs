use crate::enums::lua_type::lua_Type;
use crate::macros::lvalue::lvalue;
use crate::macros::setlvalue::setlvalue;
use crate::macros::ttisinteger::ttisinteger;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_integerclamp(
    _l: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 3
        && nresults <= 1
        && ttisinteger!(arg0)
        && ttisinteger!(args)
        && ttisinteger!(args.add(1))
    {
        let a: i64 = lvalue!(arg0);
        let rmin: i64 = lvalue!(args);
        let rmax: i64 = lvalue!(args.add(1));

        if rmin > rmax {
            return -1;
        }

        setlvalue!(
            res,
            if a < rmin {
                rmin
            } else if a > rmax {
                rmax
            } else {
                a
            }
        );
        1
    } else {
        -1
    }
}
