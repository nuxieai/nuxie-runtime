use crate::records::lua_state::lua_State;

pub unsafe fn lua_mainthread(l: *mut lua_State) -> *mut lua_State {
    (*(*l).global).mainthread
}
