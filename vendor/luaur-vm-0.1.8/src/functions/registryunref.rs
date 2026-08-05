use crate::functions::lua_h_getnum::lua_h_getnum;
use crate::macros::api_check::api_check;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::lua_refnil::LUA_REFNIL;
use crate::macros::setnvalue::setnvalue;
use crate::records::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

pub unsafe fn registryunref(
    l: *mut lua_State,
    reference: core::ffi::c_int,
    registry: *mut TValue,
    registryfree: *mut core::ffi::c_int,
) {
    luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauGcTraceUdata.get());
    if reference <= LUA_REFNIL {
        return;
    }
    let reg = hvalue!(registry);
    let slot = lua_h_getnum(reg, reference);
    api_check!(l, slot != luaO_nilobject);
    setnvalue!(slot as *mut TValue, *registryfree as f64);
    *registryfree = reference;
}
