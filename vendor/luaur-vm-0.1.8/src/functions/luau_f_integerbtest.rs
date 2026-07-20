use crate::enums::lua_type::lua_Type;
use crate::macros::lvalue::lvalue;
use crate::macros::setbvalue::setbvalue;
use crate::macros::ttisinteger::ttisinteger;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_integerbtest(
    _l: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 && ttisinteger!(arg0) {
        let mut r: u64 = lvalue!(arg0) as u64;

        for i in 2..=nparams {
            let arg = args.add((i - 2) as usize);
            if !ttisinteger!(arg) {
                return -1;
            }

            r &= lvalue!(arg) as u64;
        }

        setbvalue!(res, r != 0);
        1
    } else {
        -1
    }
}
