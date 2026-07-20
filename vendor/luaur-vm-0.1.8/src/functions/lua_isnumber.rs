use crate::functions::index_2_addr::index2addr;
use crate::macros::tonumber::tonumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_isnumber(L: *mut lua_State, idx: i32) -> i32 {
    let mut n: TValue = core::mem::zeroed();
    let mut o = index2addr(L, idx);
    tonumber!(o, &mut n) as i32
}
