use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_h_getn::lua_h_getn;
use crate::functions::lua_h_setnum::luaH_setnum;
use crate::macros::api_check::api_check;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_c_barriert::luaC_barriert;
use crate::macros::lua_refnil::LUA_REFNIL;
use crate::macros::lua_registryindex::LUA_REGISTRYINDEX;
use crate::macros::nvalue::nvalue;
use crate::macros::registry::registry;
use crate::macros::setobj_2_t::setobj2t;
use crate::macros::ttisnil::ttisnil;
use crate::records::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_ref(L: *mut lua_State, idx: c_int) -> c_int {
    api_check!(L, idx != LUA_REGISTRYINDEX);

    let mut ref_ = LUA_REFNIL;
    let g = (*L).global;
    let p: StkId = index2addr(L, idx);

    if !ttisnil!(p) {
        let reg: *mut LuaTable = hvalue!(registry!(L)) as *mut LuaTable;

        if (*g).registryfree != 0 {
            ref_ = (*g).registryfree;
        } else {
            // The dependency card for lua_h_getn shows an empty signature: pub fn lua_h_getn();
            // In Luau VM, luaH_getn(t) returns int. We transmute to the real signature.
            let lua_h_getn_real: unsafe extern "C" fn(*mut LuaTable) -> c_int =
                core::mem::transmute(lua_h_getn as *const core::ffi::c_void);
            ref_ = lua_h_getn_real(reg);
            ref_ += 1;
        }

        let slot: *mut TValue = luaH_setnum(L, reg, ref_);
        if (*g).registryfree != 0 {
            (*g).registryfree = nvalue!(slot) as c_int;
        }

        setobj2t!(L, slot, p);

        luaC_barriert!(L, reg, p);
    }

    ref_
}
