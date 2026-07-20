use crate::type_aliases::lua_state::lua_State;

pub unsafe fn lua_callbacks(L: *mut lua_State) -> *mut crate::records::lua_callbacks::LuaCallbacks {
    &mut (*(*L).global).cb as *mut crate::records::lua_callbacks::LuaCallbacks
}
