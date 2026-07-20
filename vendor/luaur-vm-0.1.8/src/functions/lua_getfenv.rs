use core::ffi::c_int;

use crate::enums::lua_type::lua_Type;
use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::macros::api_check::api_check;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::clvalue::clvalue;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::sethvalue::sethvalue;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::thvalue::thvalue;
use crate::macros::ttype::ttype;
use crate::records::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_getfenv(L: *mut lua_State, idx: c_int) {
    lua_c_threadbarrier_lapi(L);

    let o: StkId = index2addr(L, idx);
    api_check!(L, o != luaO_nilobject as StkId);

    match ttype!(o) {
        x if x == lua_Type::LUA_TFUNCTION as c_int => {
            sethvalue!(L, (*L).top, (*clvalue!(o)).env);
        }
        x if x == lua_Type::LUA_TTHREAD as c_int => {
            sethvalue!(L, (*L).top, (*thvalue!(o)).gt);
        }
        _ => {
            setnilvalue!((*L).top);
        }
    }

    api_incr_top!(L);
}
