use crate::enums::lua_type::lua_Type;
use crate::macros::lvalue::lvalue;
use crate::macros::setbvalue::setbvalue;
use crate::macros::ttisinteger::ttisinteger;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_integerule(
    _l: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttisinteger!(arg0) && ttisinteger!(args) {
        let a = lvalue!(arg0) as u64;
        let b = lvalue!(args) as u64;

        setbvalue!(res, a <= b);
        1
    } else {
        -1
    }
}
