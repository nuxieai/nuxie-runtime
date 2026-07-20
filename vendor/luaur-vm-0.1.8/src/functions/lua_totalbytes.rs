use crate::macros::api_check::api_check;
use crate::macros::lua_memory_categories::LUA_MEMORY_CATEGORIES;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_totalbytes(L: *mut lua_State, category: i32) -> usize {
    api_check!(L, category < LUA_MEMORY_CATEGORIES);
    unsafe {
        if category < 0 {
            (*(*L).global).totalbytes
        } else {
            (*(*L).global).memcatbytes[category as usize]
        }
    }
}
