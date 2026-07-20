use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_l_checkunsigned::lua_l_checkunsigned;
use crate::macros::trim::trim;
use crate::type_aliases::b_uint::b_uint;
use crate::type_aliases::lua_state::lua_State;

pub fn andaux(l: *mut lua_State) -> b_uint {
    let n = unsafe { lua_gettop(l) };
    let mut r: b_uint = !(0 as b_uint);

    for i in 1..=n {
        r &= unsafe { lua_l_checkunsigned(l, i) };
    }

    trim(r)
}
