use crate::functions::lua_v_gettable::lua_v_gettable;
use crate::macros::restorestack::restorestack;
use crate::macros::savestack::savestack;
use crate::macros::sethvalue::sethvalue;
use crate::macros::ttisnil::ttisnil;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn lua_v_getimport(
    L: *mut lua_State,
    env: *mut LuaTable,
    k: *mut TValue,
    mut res: StkId,
    id: u32,
    propagatenil: bool,
) {
    let count = id >> 30;
    LUAU_ASSERT!(count > 0);

    let id0 = ((id >> 20) & 1023) as usize;
    let id1 = ((id >> 10) & 1023) as usize;
    let id2 = (id & 1023) as usize;

    // after the first call to luaV_gettable, res may be invalid, and env may (sometimes) be garbage collected
    // we take care to not use env again and to restore res before every consecutive use
    let resp = savestack!(L, res);

    // global lookup for id0
    let mut g: TValue = core::mem::zeroed();
    sethvalue!(L, &mut g as *mut TValue, env);
    lua_v_gettable(L, &g as *const TValue, k.add(id0), res);

    // table lookup for id1
    if count < 2 {
        return;
    }

    res = restorestack!(L, resp);
    if !propagatenil || !ttisnil!(res) {
        lua_v_gettable(L, res as *const TValue, k.add(id1), res);
    }

    // table lookup for id2
    if count < 3 {
        return;
    }

    res = restorestack!(L, resp);
    if !propagatenil || !ttisnil!(res) {
        lua_v_gettable(L, res as *const TValue, k.add(id2), res);
    }
}
