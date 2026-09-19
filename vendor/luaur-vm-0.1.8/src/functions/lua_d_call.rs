use crate::functions::lua_d_callint::lua_d_callint;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

pub unsafe fn lua_d_call(
    L: *mut lua_State,
    func: StkId,
    nresults: core::ffi::c_int,
) -> crate::records::lua_exception::LuaResult<()> {
    lua_d_callint(L, func, nresults, false)
}
