use crate::functions::call_t_mres::call_t_mres;
use crate::functions::lua_g_ordererror::luaG_ordererror;
use crate::functions::lua_o_rawequal_obj::luaO_rawequalObj;
use crate::functions::lua_t_gettmbyobj::lua_t_gettmbyobj;
use crate::macros::l_isfalse::l_isfalse;
use crate::macros::ttisnil::ttisnil;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::t_value::TValue;
use crate::type_aliases::tms::TMS;

#[allow(non_snake_case)]
pub fn call_orderTM(
    L: *mut LuaState,
    p1: *const TValue,
    p2: *const TValue,
    event: TMS,
    error: bool,
) -> i32 {
    unsafe {
        let tm1 = lua_t_gettmbyobj(L, p1, event);
        let tm2;

        if ttisnil!(tm1) {
            if error {
                luaG_ordererror(L, p1, p2, event);
            }
            return -1;
        }

        tm2 = lua_t_gettmbyobj(L, p2, event);
        if luaO_rawequalObj(tm1, tm2) == 0 {
            if error {
                luaG_ordererror(L, p1, p2, event);
            }
            return -1;
        }

        call_t_mres(L, (*L).top, tm1, p1, p2);
        if l_isfalse!((*L).top) {
            0
        } else {
            1
        }
    }
}
