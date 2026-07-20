use crate::enums::lua_type::lua_Type;
use crate::macros::luai_num_2_unsigned::luai_num2unsigned;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_bnot(
    _l: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 && ttisnumber!(arg0) {
        let a1 = nvalue!(arg0);
        let mut u: u32 = 0;
        luai_num2unsigned(&mut u, a1);

        let r: u32 = !u;
        setnvalue!(res, r as f64);
        1
    } else {
        -1
    }
}
