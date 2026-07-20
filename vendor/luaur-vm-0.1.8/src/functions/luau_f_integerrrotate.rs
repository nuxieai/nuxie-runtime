use crate::enums::lua_type::lua_Type;
use crate::macros::lvalue::lvalue;
use crate::macros::setlvalue::setlvalue;
use crate::macros::ttisinteger::ttisinteger;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_integerrrotate(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttisinteger!(arg0) && ttisinteger!(_args) {
        let n: u64 = lvalue!(arg0) as u64;
        let s: u32 = (lvalue!(_args) as u64 % 64) as u32;

        let rotated: i64 = if s != 0 {
            (((n >> s) | (n << (64 - s))) as i64)
        } else {
            n as i64
        };

        setlvalue!(res, rotated);

        // Keep lua_Type import live for macro expansions / symbol expectations.
        let _ = lua_Type::LUA_TINTEGER;

        1
    } else {
        -1
    }
}
