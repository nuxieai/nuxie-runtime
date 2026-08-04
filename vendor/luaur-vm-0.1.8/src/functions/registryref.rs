use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_h_getn::lua_h_getn;
use crate::functions::lua_h_setnum::luaH_setnum;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_c_barriert::luaC_barriert;
use crate::macros::lua_refnil::LUA_REFNIL;
use crate::macros::nvalue::nvalue;
use crate::macros::setobj_2_t::setobj2t;
use crate::macros::ttisnil::ttisnil;
use crate::records::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;

pub unsafe fn registryref(
    l: *mut lua_State,
    idx: core::ffi::c_int,
    registry: *mut TValue,
    registryfree: *mut core::ffi::c_int,
) -> core::ffi::c_int {
    luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauGcTraceUdata.get());
    let mut reference = LUA_REFNIL;
    let p = index2addr(l, idx);
    if !ttisnil!(p) {
        let reg: *mut LuaTable = hvalue!(registry);
        if *registryfree != 0 {
            reference = *registryfree;
        } else {
            let getn: unsafe extern "C" fn(*mut LuaTable) -> core::ffi::c_int =
                core::mem::transmute(lua_h_getn as *const core::ffi::c_void);
            reference = getn(reg) + 1;
        }
        let slot = luaH_setnum(l, reg, reference);
        if *registryfree != 0 {
            *registryfree = nvalue!(slot) as core::ffi::c_int;
        }
        setobj2t!(l, slot, p);
        luaC_barriert!(l, reg, p);
    }
    reference
}
