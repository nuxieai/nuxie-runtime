use core::ffi::c_int;

use crate::functions::index_2_addr::index_2_addr;
pub use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luaA_toobject(L: *mut lua_State, idx: c_int) -> *const TValue {
    let p: StkId = index_2_addr(L, idx);

    if p == luaO_nilobject as StkId {
        core::ptr::null()
    } else {
        p as *const TValue
    }
}
