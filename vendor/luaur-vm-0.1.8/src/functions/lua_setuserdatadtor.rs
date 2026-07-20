use crate::macros::api_check::api_check;
use crate::macros::lua_utag_limit::LUA_UTAG_LIMIT;
use crate::type_aliases::lua_destructor::lua_Destructor;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_setuserdatadtor(L: *mut lua_State, tag: i32, dtor: lua_Destructor) {
    api_check!(L, (tag as u32) < LUA_UTAG_LIMIT as u32);
    unsafe {
        (*(*L).global).udatagc[tag as usize] = dtor;
    }
}
