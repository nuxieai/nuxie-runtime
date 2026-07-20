use crate::enums::lua_type::lua_Type;
use crate::macros::lvalue::lvalue;
use crate::macros::setlvalue::setlvalue;
use crate::macros::ttisinteger::ttisinteger;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_integercountrz(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 && ttisinteger!(arg0) {
        let n = lvalue!(arg0) as u64;

        // Rust's trailing_zeros() on a u64 is equivalent to __builtin_ctzll.
        // For n == 0, trailing_zeros() returns 64, which matches the C++ logic: (n == 0) ? 64 : __builtin_ctzll(n).
        let result = n.trailing_zeros() as i64;

        setlvalue!(res, result);

        1
    } else {
        -1
    }
}
