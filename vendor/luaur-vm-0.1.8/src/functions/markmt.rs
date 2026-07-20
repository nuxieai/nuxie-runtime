use crate::enums::lua_type::lua_Type;
use crate::enums::lua_type::LUA_T_COUNT;
use crate::macros::markobject::markobject;
use crate::type_aliases::global_state::global_State;

pub fn markmt(g: *mut global_State) {
    let mut i = 0;
    while i < (LUA_T_COUNT as i32) {
        unsafe {
            if !(*g).mt[i as usize].is_null() {
                markobject!(g, (*g).mt[i as usize]);
            }
        }
        i += 1;
    }
}
