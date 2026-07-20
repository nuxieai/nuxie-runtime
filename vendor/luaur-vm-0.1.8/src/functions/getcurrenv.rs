use crate::enums::lua_type::lua_Type;
use crate::macros::curr_func::curr_func;
use crate::records::lua_state::lua_State;
use crate::records::lua_table::LuaTable;

pub unsafe fn getcurrenv(l: *mut lua_State) -> *mut LuaTable {
    if (*l).ci == (*l).base_ci {
        (*l).gt
    } else {
        (*curr_func!(l)).env
    }
}
