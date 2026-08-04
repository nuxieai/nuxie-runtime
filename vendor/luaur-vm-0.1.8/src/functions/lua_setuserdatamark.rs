use crate::records::lua_state::lua_State;
use crate::type_aliases::lua_userdata_mark::lua_UserdataMark;

pub unsafe fn lua_setuserdatamark(
    l: *mut lua_State,
    tag: core::ffi::c_int,
    markfn: lua_UserdataMark,
) {
    luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauGcTraceUdata.get());
    crate::macros::api_check::api_check!(
        l,
        (tag as u32) < crate::macros::lua_utag_limit::LUA_UTAG_LIMIT as u32
    );
    (*(*l).global).udatamark[tag as usize] = markfn;
}
