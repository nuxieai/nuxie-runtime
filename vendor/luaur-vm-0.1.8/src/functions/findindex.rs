use crate::functions::arrayindex::arrayindex;
use crate::functions::lua_o_rawequal_key::luaO_rawequalKey;
use crate::functions::mainposition::mainposition;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::stk_id::StkId;

use crate::macros::cast_int::cast_int;
use crate::macros::gcvalue::gcvalue;
use crate::macros::gkey::gkey;
use crate::macros::gnode::gnode;
use crate::macros::iscollectable::iscollectable;
use crate::macros::nvalue::nvalue;
use crate::macros::ttisnil::ttisnil;
use crate::macros::ttisnumber::ttisnumber;
use crate::macros::ttype::ttype;

use crate::records::lua_node::LuaNode;

#[allow(non_snake_case)]
pub unsafe fn findindex(L: *mut lua_State, t: *mut LuaTable, key: StkId) -> i32 {
    let mut i: i32;

    if ttisnil!(key) {
        return -1; // first iteration
    }

    i = if ttisnumber!(key) {
        arrayindex(nvalue!(key))
    } else {
        -1
    };

    if i > 0 && i <= (*t).sizearray {
        return i - 1; // yes; that's the index (corrected to C)
    } else {
        let mut n: *mut LuaNode = mainposition(t, key as *const _);

        loop {
            // check whether `key' is somewhere in the chain
            // key may be dead already, but it is ok to use it in `next'
            if luaO_rawequalKey(gkey!(n) as *const _, key as *const _) != 0
                || (ttype!(gkey!(n)) == crate::enums::lua_type::lua_Type::LUA_TDEADKEY as i32
                    && iscollectable!(key)
                    && gcvalue!(gkey!(n)) == gcvalue!(key))
            {
                i = cast_int!(n.offset_from(gnode!(t, 0)) as i32);
                // hash elements are numbered after array ones
                return i + (*t).sizearray;
            }

            // gnext(n) is defined as ((n)->key.next) in ltable.h
            let next_offset = (*n).key.next();

            if next_offset == 0 {
                break;
            }
            n = n.offset(next_offset as isize);
        }

        crate::functions::lua_g_runerror_l::lua_g_runerror_l(
            L,
            core::ptr::null(),
            format_args!("invalid key to 'next'"),
        );
    }
}
