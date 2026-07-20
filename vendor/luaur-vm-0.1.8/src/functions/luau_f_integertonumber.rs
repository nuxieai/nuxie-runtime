use crate::enums::lua_type::lua_Type;
use crate::macros::cast_num::cast_num;
use crate::macros::lvalue::lvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisinteger::ttisinteger;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_integertonumber(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    // Keep lua_Type import live for macro expansions / symbol expectations.
    let _ = lua_Type::LUA_TINTEGER;

    if nparams >= 1 && nresults <= 1 && ttisinteger!(arg0) {
        setnvalue!(res, cast_num!(lvalue!(arg0)));
        1
    } else {
        -1
    }
}
