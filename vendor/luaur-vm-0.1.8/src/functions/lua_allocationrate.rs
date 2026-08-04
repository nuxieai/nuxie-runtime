use crate::records::lua_state::lua_State;

pub unsafe fn lua_allocationrate(l: *mut lua_State) -> i64 {
    crate::functions::lua_c_allocationrate::luaC_allocationrate(l)
}
