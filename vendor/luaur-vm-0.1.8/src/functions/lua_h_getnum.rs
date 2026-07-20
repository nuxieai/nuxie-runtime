use crate::functions::hashnum::hashnum;
use crate::functions::lua_a_toobject::luaO_nilobject;
use crate::macros::cast_num::cast_num;
use crate::macros::dummynode::luaH_dummynode;
use crate::macros::gkey::{gkey, gval};
use crate::macros::luai_numeq::luai_numeq;
use crate::macros::nvalue::nvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_h_getnum(t: *mut LuaTable, key: core::ffi::c_int) -> *const TValue {
    // (1 <= key && key <= t->sizearray)
    if (key as core::ffi::c_uint).wrapping_sub(1) < (*t).sizearray as core::ffi::c_uint {
        return (*t).array.add((key - 1) as usize);
    } else if (*t).node != &luaH_dummynode as *const _ as *mut _ {
        let nk = cast_num!(key);
        let mut n = hashnum(t, nk);

        loop {
            // check whether `key' is somewhere in the chain
            if ttisnumber!(gkey!(n)) && luai_numeq(nvalue!(gkey!(n)), nk) {
                return gval!(n); // that's it
            }

            // gnext(n) is defined as ((n)->key.next) in ltable.h
            let next_offset = (*n).key.next();

            if next_offset == 0 {
                break;
            }
            n = n.offset(next_offset as isize);
        }
        return luaO_nilobject;
    } else {
        return luaO_nilobject;
    }
}
