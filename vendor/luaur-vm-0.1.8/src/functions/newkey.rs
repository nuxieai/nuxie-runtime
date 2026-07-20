use crate::functions::arrayornewkey::arrayornewkey;
use crate::functions::getfreepos::getfreepos;
use crate::functions::mainposition::mainposition;
use crate::functions::rehash::rehash;
use crate::macros::dummynode::dummynode;
use crate::macros::gkey::gval;
use crate::macros::lua_c_barriert::luaC_barriert;
use crate::macros::nvalue::nvalue;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::ttisnil::ttisnil;
use crate::macros::ttisnumber::ttisnumber;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[inline]
unsafe fn getnodekey_direct(obj: *mut TValue, node: *const LuaNode) {
    (*obj).value = (*node).key.value;
    core::ptr::copy_nonoverlapping((*node).key.extra.as_ptr(), (*obj).extra.as_mut_ptr(), 1);
    (*obj).tt = (*node).key.tt();
}

#[inline]
unsafe fn setnodekey_direct(node: *mut LuaNode, obj: *const TValue) {
    (*node).key.value = (*obj).value;
    core::ptr::copy_nonoverlapping((*obj).extra.as_ptr(), (*node).key.extra.as_mut_ptr(), 1);
    (*node).key.set_tt((*obj).tt);
}

#[allow(non_snake_case)]
pub unsafe fn newkey(l: *mut lua_State, t: *mut LuaTable, key: *const TValue) -> *mut TValue {
    if ttisnumber!(key) && nvalue!(key) == ((*t).sizearray + 1) as f64 {
        rehash(l, t, key);
        return arrayornewkey(l, t, key);
    }

    let mut mp = mainposition(t, key);
    if !ttisnil!(gval!(mp)) || mp == dummynode as *mut LuaNode {
        let n = getfreepos(t);
        if n.is_null() {
            rehash(l, t, key);
            return arrayornewkey(l, t, key);
        }

        LUAU_ASSERT!(n != dummynode as *mut LuaNode);

        let mut mk = TValue::default();
        getnodekey_direct(core::ptr::addr_of_mut!(mk), mp);
        let mut othern = mainposition(t, core::ptr::addr_of!(mk));

        if othern != mp {
            while othern.offset((*othern).key.next() as isize) != mp {
                othern = othern.offset((*othern).key.next() as isize);
            }

            (*othern).key.set_next(n.offset_from(othern) as i32);
            *n = *mp;

            if (*mp).key.next() != 0 {
                (*n).key
                    .set_next((*n).key.next() + mp.offset_from(n) as i32);
                (*mp).key.set_next(0);
            }

            setnilvalue!(gval!(mp));
        } else {
            if (*mp).key.next() != 0 {
                (*n).key
                    .set_next(mp.offset((*mp).key.next() as isize).offset_from(n) as i32);
            } else {
                LUAU_ASSERT!((*n).key.next() == 0);
            }

            (*mp).key.set_next(n.offset_from(mp) as i32);
            mp = n;
        }
    }

    setnodekey_direct(mp, key);
    luaC_barriert!(l, t, key);
    LUAU_ASSERT!(ttisnil!(gval!(mp)));
    gval!(mp)
}
