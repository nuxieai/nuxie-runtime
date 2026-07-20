use crate::enums::lua_type::lua_Type;
use crate::macros::l_isfalse::l_isfalse;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_assert(
    _L: *mut lua_State,
    _res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    // The macro l_isfalse! depends on lua_Type being in scope in the caller's context
    // because it expands to comparisons against lua_Type::LUA_TNIL etc.
    let _ = lua_Type::LUA_TNIL;

    if nparams >= 1 && nresults == 0 && !l_isfalse!(arg0) {
        return 0;
    }

    -1
}
