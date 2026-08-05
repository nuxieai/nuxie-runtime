use crate::functions::lua_h_get::lua_h_get;
use crate::macros::hvalue::hvalue;
use crate::macros::setobj_2_s::setobj_2_s;
use crate::macros::ttistable::ttistable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_rawget(
    L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttistable!(arg0) {
        setobj_2_s!(L, res, lua_h_get(hvalue!(arg0), args));
        return 1;
    }

    -1
}
