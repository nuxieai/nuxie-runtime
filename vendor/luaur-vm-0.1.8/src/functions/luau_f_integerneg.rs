use crate::enums::lua_type::lua_Type;
use crate::macros::lvalue::lvalue;
use crate::macros::setlvalue::setlvalue;
use crate::macros::ttisinteger::ttisinteger;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_integerneg(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 && ttisinteger!(arg0) {
        // C++: (int64_t)(~(uint64_t)lvalue(arg0) + 1)
        // This is the standard two's complement negation: -x = !x + 1
        let val = lvalue!(arg0) as u64;
        let neg_val = (!val).wrapping_add(1) as i64;
        setlvalue!(res, neg_val);
        1
    } else {
        -1
    }
}
