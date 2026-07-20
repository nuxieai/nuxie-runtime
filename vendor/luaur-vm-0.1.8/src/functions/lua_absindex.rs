use core::ffi::c_int;

use crate::macros::api_check::api_check;
use crate::macros::cast_int::cast_int;
use crate::macros::lua_ispseudo::lua_ispseudo;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_absindex(L: *mut lua_State, idx: c_int) -> c_int {
    let top_minus_base = unsafe { (*L).top.offset_from((*L).base) };

    api_check!(
        L,
        (idx > 0 && idx <= unsafe { cast_int!(top_minus_base) })
            || (idx < 0 && -idx <= unsafe { cast_int!(top_minus_base) })
            || lua_ispseudo(idx)
    );

    if idx > 0 || lua_ispseudo(idx) {
        idx
    } else {
        cast_int!(unsafe { (*L).top.offset_from((*L).base) }) + idx + 1
    }
}
