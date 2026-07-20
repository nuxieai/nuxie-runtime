use crate::enums::lua_type::lua_Type;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_max(
    _l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttisnumber!(arg0) && ttisnumber!(args) {
        let a1 = nvalue!(arg0);
        let a2 = nvalue!(args);

        let mut r = if a2 > a1 { a2 } else { a1 };

        for i in 3..=nparams {
            let arg_i = args.add((i - 2) as usize);

            if !ttisnumber!(arg_i) {
                return -1;
            }

            let a = nvalue!(arg_i);
            r = if a > r { a } else { r };
        }

        setnvalue!(res, r);
        1
    } else {
        -1
    }
}
