use crate::enums::lua_type::lua_Type;
use crate::macros::luai_num_2_unsigned::luai_num2unsigned;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_band(
    _l: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttisnumber!(arg0) && ttisnumber!(args) {
        let a1 = nvalue!(arg0);
        let a2 = nvalue!(args);

        let mut u1: u32 = 0;
        let mut u2: u32 = 0;
        luai_num2unsigned(&mut u1, a1);
        luai_num2unsigned(&mut u2, a2);

        let mut r: u32 = u1 & u2;

        for i in 3..=nparams {
            let arg_ptr = args.add((i - 2) as usize);
            if !ttisnumber!(arg_ptr) {
                return -1;
            }

            let a = nvalue!(arg_ptr);
            let mut u: u32 = 0;
            luai_num2unsigned(&mut u, a);

            r &= u;
        }

        setnvalue!(res, r as f64);
        1
    } else {
        -1
    }
}
