use crate::functions::lua_l_checkunsigned::lua_l_checkunsigned;
use crate::functions::lua_pushunsigned::lua_pushunsigned;
use crate::macros::trim::trim;
use crate::type_aliases::b_uint::b_uint;
use crate::type_aliases::lua_state::lua_State;

use crate::macros::nbits::NBITS;

pub fn b_rot(l: *mut lua_State, mut i: core::ffi::c_int) -> core::ffi::c_int {
    let mut r: b_uint = lua_l_checkunsigned(l, 1);

    // i = i % NBITS (avoid undefined shift when i == 0)
    i &= (NBITS - 1) as core::ffi::c_int;

    r = trim(r);
    if i != 0 {
        let i_u = i as u32;
        r = (r << i_u) | (r >> (NBITS as u32 - i_u));
    }

    lua_pushunsigned(l, trim(r));
    1
}
