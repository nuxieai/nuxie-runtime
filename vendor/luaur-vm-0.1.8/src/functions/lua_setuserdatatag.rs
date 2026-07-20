use crate::functions::index_2_addr::index2addr;
use crate::macros::api_check::api_check;
use crate::macros::lua_utag_limit::LUA_UTAG_LIMIT;
use crate::macros::ttisuserdata::ttisuserdata;
use crate::records::udata::Udata;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_setuserdatatag(L: *mut lua_State, idx: core::ffi::c_int, tag: core::ffi::c_int) {
    api_check!(L, (tag as u32) < LUA_UTAG_LIMIT as u32);
    let o: StkId = index2addr(L, idx);
    api_check!(L, ttisuserdata!(o));
    let u = core::ptr::addr_of_mut!((*(*o).value.gc).u) as *mut Udata;
    (*u).tag = tag as u8;
}
