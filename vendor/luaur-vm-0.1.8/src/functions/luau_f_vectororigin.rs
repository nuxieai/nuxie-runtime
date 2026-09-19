use crate::macros::setvvalue::setvvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_vectororigin(
    l: *mut lua_State,
    res: StkId,
    _arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    _nparams: core::ffi::c_int,
) -> crate::records::lua_exception::LuaResult<core::ffi::c_int> {
    if nresults <= 1 {
        setvvalue!(l, res, 0.0, 0.0, 0.0, 0.0);
        return Ok(1);
    }

    Ok(-1)
}
