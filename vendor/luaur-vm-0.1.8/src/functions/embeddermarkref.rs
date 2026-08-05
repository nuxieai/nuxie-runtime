use crate::functions::lua_h_getnum::lua_h_getnum;
use crate::functions::reallymarkobject::reallymarkobject;
use crate::macros::gcvalue::gcvalue;
use crate::macros::hvalue::hvalue;
use crate::macros::iscollectable::iscollectable;
use crate::macros::iswhite::iswhite;
use crate::records::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe extern "C" fn embeddermarkref(l: *mut lua_State, reference: core::ffi::c_int) {
    luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauGcTraceUdata.get());
    if reference <= crate::macros::lua_refnil::LUA_REFNIL {
        return;
    }
    let g = (*l).global;
    let wt = hvalue!(core::ptr::addr_of!((*g).weakregistry));
    let slot = lua_h_getnum(wt, reference);
    if iscollectable!(slot) && iswhite!(gcvalue!(slot)) {
        reallymarkobject(g, gcvalue!(slot));
    }
}
