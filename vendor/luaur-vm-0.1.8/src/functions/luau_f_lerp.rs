use crate::enums::lua_type::lua_Type;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_lerp(
    _L: *mut lua_State,
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
        let a = nvalue!(arg0);
        let b = nvalue!(args);
        let t = nvalue!(args.add(1));

        let r = if t == 1.0 { b } else { a + (b - a) * t };

        setnvalue!(res, r);
        1
    } else {
        -1
    }
}
