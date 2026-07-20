use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::macros::ttisthread::ttisthread;
use crate::records::lua_state::lua_State as LuaStateRecord;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_tothread(L: *mut lua_State, idx: c_int) -> *mut lua_State {
    let o: StkId = index2addr(L, idx);

    if !ttisthread!(o) {
        core::ptr::null_mut()
    } else {
        core::ptr::addr_of_mut!((*(*o).value.gc).th) as *mut LuaStateRecord
    }
}
