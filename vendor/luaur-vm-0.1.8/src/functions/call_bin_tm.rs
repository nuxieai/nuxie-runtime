use crate::functions::call_t_mres::call_t_mres;
use crate::functions::lua_t_gettmbyobj::lua_t_gettmbyobj;
use crate::macros::ttisnil::ttisnil;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;
use crate::type_aliases::tms::TMS;

#[allow(non_snake_case)]
pub fn call_bin_tm(
    L: *mut LuaState,
    p1: *const TValue,
    p2: *const TValue,
    res: StkId,
    event: TMS,
) -> i32 {
    unsafe {
        let mut tm = lua_t_gettmbyobj(L, p1, event); // try first operand
        if ttisnil!(tm) {
            tm = lua_t_gettmbyobj(L, p2, event); // try second operand
        }
        if ttisnil!(tm) {
            return 0;
        }
        call_t_mres(L, res, tm, p1, p2);
        1
    }
}
