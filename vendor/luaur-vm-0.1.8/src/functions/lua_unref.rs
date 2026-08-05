use core::ffi::c_int;

use crate::functions::lua_h_getnum::lua_h_getnum;
use crate::macros::api_check::api_check;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::lua_refnil::LUA_REFNIL;
use crate::macros::registry::registry;
use crate::macros::setnvalue::setnvalue;
use crate::records::global_state::global_State;
use crate::records::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_unref(L: *mut lua_State, ref_: c_int) -> c_int {
    if luaur_common::FFlag::LuauGcTraceUdata.get() {
        let g = (*L).global;
        crate::functions::registryunref::registryunref(
            L,
            ref_,
            core::ptr::addr_of_mut!((*g).registry),
            core::ptr::addr_of_mut!((*g).registryfree),
        );
        return crate::macros::lua_noref::LUA_NOREF;
    }

    if ref_ <= LUA_REFNIL {
        return crate::macros::lua_noref::LUA_NOREF;
    }

    let g: *mut global_State = (*L).global;

    // The hvalue! macro expects a pointer to a TValue.
    // registry!(L) returns &(*(*L).global).registry, which is a &TValue.
    let reg_tvalue_ptr: *const TValue = registry!(L);
    let reg: *mut LuaTable = hvalue!(reg_tvalue_ptr) as *const _ as *mut LuaTable;

    let slot: *const TValue = lua_h_getnum(reg, ref_);

    api_check!(L, slot != luaO_nilobject);

    // similar to how 'luaH_setnum' makes non-nil slot value mutable
    let mutable_slot = slot as *mut TValue;

    // NB: no barrier needed because value isn't collectable (it's a number)
    setnvalue!(mutable_slot, (*g).registryfree as f64);

    (*g).registryfree = ref_;
    crate::macros::lua_noref::LUA_NOREF
}
