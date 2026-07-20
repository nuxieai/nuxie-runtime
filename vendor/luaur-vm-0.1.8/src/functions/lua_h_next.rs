use crate::functions::findindex::findindex;
use crate::macros::cast_num::cast_num;
use crate::macros::getnodekey::getnodekey;
use crate::macros::gnext::gnext;
use crate::macros::gnode::gnode;
use crate::macros::setnvalue::setnvalue;
use crate::macros::setobj_2_s::setobj_2_s;
use crate::macros::sizenode::sizenode;
use crate::macros::ttisnil::ttisnil;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::stk_id::StkId;

// Required by getnodekey macro expansion
use crate::macros::checkliveness::checkliveness;
use crate::records::lua_node::LuaNode;
use crate::records::lua_t_value::lua_TValue;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_h_next(L: *mut lua_State, t: *mut LuaTable, key: StkId) -> i32 {
    let mut i = findindex(L, t, key);

    i += 1;

    // try first array part
    while i < (*t).sizearray {
        let e = (*t).array.add(i as usize);
        if !ttisnil!(e) {
            setnvalue!(key, cast_num!(i + 1));
            setobj_2_s!(L, key.add(1), e);
            return 1;
        }
        i += 1;
    }

    // then hash part
    let mut k = i - (*t).sizearray;
    let size = sizenode!(t);
    while k < size {
        let n = gnode!(t, k);
        // gval(n) in C++ is (&(n)->val). In our Rust macros, gval is not provided as a standalone macro
        // but the logic is usually just the address of the val field.
        let val = core::ptr::addr_of_mut!((*n).val);
        if !ttisnil!(val) {
            getnodekey!(L, key, n);
            setobj_2_s!(L, key.add(1), val);
            return 1;
        }
        k += 1;
    }

    0 // no more elements
}
