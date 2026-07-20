use crate::functions::fieldargs::fieldargs;
use crate::functions::lua_l_checkunsigned::lua_l_checkunsigned;
use crate::functions::lua_pushunsigned::lua_pushunsigned;
use crate::macros::mask::mask;
use crate::type_aliases::b_uint::b_uint;
use crate::type_aliases::lua_state::lua_State;

pub fn b_replace(l: *mut lua_State) -> core::ffi::c_int {
    let mut w: core::ffi::c_int = 0;
    let r: b_uint = unsafe { lua_l_checkunsigned(l, 1) };
    let mut v: b_uint = unsafe { lua_l_checkunsigned(l, 2) };
    let f: core::ffi::c_int = unsafe { fieldargs(l, 3, &mut w) };
    let m: b_uint = mask(w);
    v &= m;
    let r = (r & !(m << f)) | (v << f);
    lua_pushunsigned(l, r);
    1
}
