use crate::functions::lua_t_objtypenamestr::lua_t_objtypenamestr;
use crate::macros::setsvalue::setsvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_typeof(
    l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 {
        let ttname = lua_t_objtypenamestr(l, arg0);

        setsvalue!(l, res, ttname);
        return 1;
    }

    -1
}
