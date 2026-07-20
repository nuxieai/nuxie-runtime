use crate::functions::fieldargs::fieldargs;
use crate::functions::lua_l_checkunsigned::lua_l_checkunsigned;
use crate::functions::lua_pushunsigned::lua_pushunsigned;
use crate::macros::mask::mask;
use crate::type_aliases::b_uint::b_uint;
use crate::type_aliases::lua_state::lua_State;

pub fn b_extract(l: *mut lua_State) -> core::ffi::c_int {
    let mut w: core::ffi::c_int = 0;
    let r: b_uint = unsafe { lua_l_checkunsigned(l, 1) };
    let f: core::ffi::c_int = unsafe { fieldargs(l, 2, &mut w) };
    let r = (r >> f) & mask(w);
    lua_pushunsigned(l, r);
    1
}
