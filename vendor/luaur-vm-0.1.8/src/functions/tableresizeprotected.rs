use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_rawrunprotected_ldo::luaD_rawrunprotected;
use crate::methods::call_context_run_lgc_alt_c::*;
use crate::records::call_context_lgc_alt_c::CallContext;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn tableresizeprotected(l: *mut lua_State, t: *mut LuaTable, nhsize: c_int) {
    let mut ctx = CallContext { t, nhsize };
    let status = luaD_rawrunprotected(
        l,
        Some(CallContext::run),
        core::ptr::addr_of_mut!(ctx) as *mut core::ffi::c_void,
    );
    LUAU_ASSERT!(
        status == lua_Status::LUA_OK as c_int || status == lua_Status::LUA_ERRMEM as c_int
    );
}
