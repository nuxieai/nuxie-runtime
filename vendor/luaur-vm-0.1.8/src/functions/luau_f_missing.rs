use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_missing(
    _L: *mut lua_State,
    _res: StkId,
    _arg0: *mut TValue,
    _nresults: core::ffi::c_int,
    _args: StkId,
    _nparams: core::ffi::c_int,
) -> crate::records::lua_exception::LuaResult<core::ffi::c_int> {
    Ok(-1)
}
