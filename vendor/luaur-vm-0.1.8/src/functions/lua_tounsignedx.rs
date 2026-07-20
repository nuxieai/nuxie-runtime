use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::macros::luai_num_2_unsigned::luai_num2unsigned;
use crate::macros::nvalue::nvalue;
use crate::macros::tonumber::tonumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_tounsignedx(L: *mut lua_State, idx: c_int, isnum: *mut c_int) -> u32 {
    let mut n: TValue = core::mem::zeroed();
    // The tonumber! macro may reassign the pointer if it needs to point to the converted temporary.
    let mut o = index2addr(L, idx);

    if tonumber!(o, &mut n) {
        let mut res: u32 = 0;
        let num = nvalue!(o);
        luai_num2unsigned(&mut res, num);

        if !isnum.is_null() {
            *isnum = 1;
        }
        res
    } else {
        if !isnum.is_null() {
            *isnum = 0;
        }
        0
    }
}
