use core::ffi::c_int;

use crate::functions::index_2_addr::index_2_addr;
use crate::macros::nvalue::nvalue;
use crate::macros::tonumber::tonumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_tonumberx(L: *mut lua_State, idx: c_int, isnum: *mut c_int) -> f64 {
    let mut n: TValue = core::mem::zeroed();
    let mut o = index_2_addr(L, idx) as *const TValue;

    if tonumber!(o, &mut n) {
        if !isnum.is_null() {
            *isnum = 1;
        }
        nvalue!(o)
    } else {
        if !isnum.is_null() {
            *isnum = 0;
        }
        0.0
    }
}
