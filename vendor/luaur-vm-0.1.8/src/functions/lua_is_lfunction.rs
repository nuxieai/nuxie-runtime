use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::macros::clvalue::clvalue;
use crate::macros::ttype::ttype;
use crate::records::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_is_lfunction(L: *mut lua_State, idx: c_int) -> c_int {
    let o: StkId = index2addr(L, idx);

    if ttype!(o) == crate::enums::lua_type::lua_Type::LUA_TFUNCTION as c_int
        && (*clvalue!(o)).isC == 0
    {
        1
    } else {
        0
    }
}
