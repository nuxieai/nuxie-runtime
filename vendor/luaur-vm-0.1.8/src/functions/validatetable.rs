use crate::enums::lua_type::lua_Type;
use crate::functions::validateobjref::validateobjref;
use crate::functions::validateref::validateref;
use crate::macros::gkey::gkey;
use crate::macros::gkey::gval;
use crate::records::gc_object::GCObject;
use crate::records::global_state::global_State;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn validatetable(g: *mut global_State, h: *mut LuaTable) {
    let sizenode = 1 << (*h).lsizenode;

    LUAU_ASSERT!((*h).union.lastfree as i32 <= sizenode);

    let h_gco = h as *mut GCObject;

    if !(*h).metatable.is_null() {
        validateobjref(g, h_gco, (*h).metatable as *mut GCObject);
    }

    for i in 0..(*h).sizearray {
        validateref(g, h_gco, (*h).array.add(i as usize));
    }

    for i in 0..sizenode {
        let n: *mut LuaNode = (*h).node.add(i as usize);

        // ttype(gkey(n)) -> (*gkey!(n)).tt()
        // ttisnil(gval(n)) -> crate::macros::ttisnil::ttisnil!(gval!(n))
        LUAU_ASSERT!(
            (*gkey!(n)).tt() != lua_Type::LUA_TDEADKEY as i32
                || crate::macros::ttisnil::ttisnil!(gval!(n))
        );

        // gnext(n) -> (*n).key.next()
        let next_val = (*n).key.next();
        LUAU_ASSERT!(i + next_val >= 0 && i + next_val < sizenode);

        if !crate::macros::ttisnil::ttisnil!(gval!(n)) {
            let mut k: TValue = core::mem::zeroed();
            k.tt = (*gkey!(n)).tt();
            k.value = (*gkey!(n)).value;

            validateref(g, h_gco, &mut k as *mut TValue);
            validateref(g, h_gco, gval!(n));
        }
    }
}
