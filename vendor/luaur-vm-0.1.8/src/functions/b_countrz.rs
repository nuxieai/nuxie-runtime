use crate::functions::lua_l_checkunsigned::lua_l_checkunsigned;
use crate::functions::lua_pushunsigned::lua_pushunsigned;
use crate::type_aliases::b_uint::b_uint;
use crate::type_aliases::lua_state::lua_State;

const NBITS: i32 = 32;

pub fn b_countrz(l: *mut lua_State) -> core::ffi::c_int {
    let v = unsafe { lua_l_checkunsigned(l, 1) } as b_uint;

    let mut r: b_uint = NBITS as b_uint;
    for i in 0..NBITS {
        if (v & (1u32 << i)) != 0 {
            r = i as b_uint;
            break;
        }
    }

    unsafe {
        lua_pushunsigned(l, r);
    }
    1
}
