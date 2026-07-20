use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_camel_case_types)]
pub type luau_FastFunction = Option<
    unsafe fn(
        L: *mut crate::type_aliases::lua_state::lua_State,
        res: StkId,
        arg0: *mut TValue,
        nresults: core::ffi::c_int,
        args: StkId,
        nparams: core::ffi::c_int,
    ) -> core::ffi::c_int,
>;

pub type LuauFastFunction = luau_FastFunction;
