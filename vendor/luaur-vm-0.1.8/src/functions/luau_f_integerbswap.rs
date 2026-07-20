use crate::enums::lua_type::lua_Type;
use crate::macros::lvalue::lvalue;
use crate::macros::setlvalue::setlvalue;
use crate::macros::ttisinteger::ttisinteger;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_integerbswap(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 && ttisinteger!(arg0) {
        let a = lvalue!(arg0) as u64;

        // Manual byte swap implementation for a 64-bit integer
        let swapped = (a >> 56)
            | ((a & 0x00FF000000000000) >> 40)
            | ((a & 0x0000FF0000000000) >> 24)
            | ((a & 0x000000FF00000000) >> 8)
            | ((a & 0x00000000FF000000) << 8)
            | ((a & 0x0000000000FF0000) << 24)
            | ((a & 0x000000000000FF00) << 40)
            | ((a & 0x00000000000000FF) << 56);

        setlvalue!(res, swapped as i64);

        // Ensure lua_Type is used to prevent potential linker issues with unused enum references in macros
        let _ = lua_Type::LUA_TINTEGER;

        1
    } else {
        -1
    }
}
