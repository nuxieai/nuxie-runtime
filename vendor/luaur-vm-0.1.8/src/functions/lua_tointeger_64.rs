use crate::functions::index_2_addr::index2addr;
use crate::macros::lvalue::lvalue;
use crate::macros::ttisinteger::ttisinteger;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_tointeger_64(L: *mut lua_State, idx: i32, isinteger: *mut i32) -> i64 {
    let o = index2addr(L, idx);
    if ttisinteger!(o) {
        if !isinteger.is_null() {
            *isinteger = 1;
        }
        lvalue!(o)
    } else {
        if !isinteger.is_null() {
            *isinteger = 0;
        }
        0
    }
}
