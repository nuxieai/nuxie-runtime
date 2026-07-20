use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::macros::api_check::api_check;
use crate::macros::api_update_top::api_update_top;
use crate::macros::getnodekey::getnodekey;
use crate::macros::hvalue::hvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::setobj_2_s::setobj2s;
use crate::macros::ttisnil::ttisnil;
use crate::macros::ttistable::ttistable;
use crate::records::lua_node::LuaNode;
use crate::records::lua_state::lua_State;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_rawiter(
    L: *mut lua_State,
    idx: core::ffi::c_int,
    iter: core::ffi::c_int,
) -> core::ffi::c_int {
    lua_c_threadbarrier_lapi(L);

    let t: StkId = index2addr(L, idx);
    api_check!(L, ttistable!(t));
    api_check!(L, iter >= 0);

    let h: *mut LuaTable = hvalue!(t);
    let sizearray = (*h).sizearray;

    // first we advance iter through the array portion
    let mut iter = iter;
    while (iter as u32) < (sizearray as u32) {
        let e: *mut TValue = (*h).array.add(iter as usize);
        if !ttisnil!(e) {
            let top: StkId = (*L).top;
            setnvalue!(top.add(0), (iter + 1) as f64);
            setobj2s!(L, top.add(1), e);
            api_update_top!(L, top.add(2));
            return iter + 1;
        }
        iter += 1;
    }

    let sizenode = 1 << (*h).lsizenode;

    // then we advance iter through the hash portion
    while ((iter - sizearray) as u32) < (sizenode as u32) {
        let n: *mut LuaNode = (*h).node.add((iter - sizearray) as usize);
        let val = core::ptr::addr_of_mut!((*n).val);
        if !ttisnil!(val) {
            let top: StkId = (*L).top;
            getnodekey!(L, top.add(0), n);
            setobj2s!(L, top.add(1), val);
            api_update_top!(L, top.add(2));
            return iter + 1;
        }
        iter += 1;
    }

    // traversal finished
    -1
}
