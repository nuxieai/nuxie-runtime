use crate::macros::lua_utag_limit::LUA_UTAG_LIMIT;
use crate::macros::markobject::markobject;
use crate::type_aliases::global_state::global_State;

pub unsafe fn marktaggetmt(g: *mut global_State) {
    for i in 0..LUA_UTAG_LIMIT as usize {
        if !(*g).udatamt[i].is_null() {
            markobject!(g, (*g).udatamt[i]);
        }
    }
}
