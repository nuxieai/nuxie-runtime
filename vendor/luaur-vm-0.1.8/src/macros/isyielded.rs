use crate::enums::lua_status::lua_Status;
use crate::macros::scheduled_reentry::SCHEDULED_REENTRY;
use crate::records::lua_state::lua_State;

pub const fn isyielded(L: *mut lua_State) -> bool {
    let status = unsafe { (*L).status as i32 };
    status == lua_Status::LUA_YIELD as i32
        || status == lua_Status::LUA_BREAK as i32
        || status == SCHEDULED_REENTRY
}
