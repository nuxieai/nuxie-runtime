use crate::functions::index_2_addr::index_2_addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_h_clone::lua_h_clone;
use crate::macros::api_check::api_check;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::sethvalue::sethvalue;
use crate::macros::ttistable::ttistable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_clonetable(
    L: *mut lua_State,
    idx: core::ffi::c_int,
) -> crate::records::lua_exception::LuaResult<()> {
    luaC_checkGC!(L);
    lua_c_threadbarrier_lapi(L);

    crate::ensure_stack!(L, 1);
    // The C++ source:
    // StkId t = index2addr(L, idx);
    // api_check(L, ttistable(t));
    // LuaTable* tt = luaH_clone(L, hvalue(t));
    // sethvalue(L, L->top, tt);
    // api_incr_top(L);

    let t: StkId = index_2_addr(L, idx);

    api_check!(L, ttistable!(t));

    let tt: *mut LuaTable = lua_h_clone(L, hvalue!(t))?;

    sethvalue!(L, (*L).top, tt);
    api_incr_top!(L);
    Ok(())
}
