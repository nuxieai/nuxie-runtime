use crate::enums::lua_type::lua_Type;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_clamp(
    _l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 3
        && nresults <= 1
        && ttisnumber!(arg0)
        && ttisnumber!(args)
        && ttisnumber!(args.add(1))
    {
        let v = nvalue!(arg0);
        let min = nvalue!(args);
        let max = nvalue!(args.add(1));

        if min <= max {
            let r = if v < min { min } else { v };
            let r = if r > max { max } else { r };

            setnvalue!(res, r);
            return 1;
        }
    }

    -1
}
