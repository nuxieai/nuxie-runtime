use crate::functions::resize::resize;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;

pub fn lua_h_resizehash(L: *mut lua_State, t: *mut LuaTable, nhsize: core::ffi::c_int) {
    unsafe {
        resize(L, t, (*t).sizearray, nhsize);
    }
}
