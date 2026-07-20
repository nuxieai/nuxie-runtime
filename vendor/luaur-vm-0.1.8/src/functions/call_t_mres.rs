use crate::functions::lua_d_call::lua_d_call;
use crate::macros::lua_d_checkstack::luaD_checkstack;
use crate::macros::restorestack::restorestack;
use crate::macros::savestack::savestack;
use crate::macros::setobj_2_s::setobj2s;

use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub unsafe fn call_t_mres(
    L: *mut LuaState,
    mut res: StkId,
    f: *const TValue,
    p1: *const TValue,
    p2: *const TValue,
) -> StkId {
    let result = savestack!(L, res);

    LUAU_ASSERT!((*L).top.offset(3) < (*L).stack.add((*L).stacksize as usize));

    setobj2s!(L, (*L).top, f);
    setobj2s!(L, (*L).top.add(1), p1);
    setobj2s!(L, (*L).top.add(2), p2);

    luaD_checkstack!(L, 3);
    (*L).top = (*L).top.add(3);

    lua_d_call(L, (*L).top.offset(-3), 1);

    res = restorestack!(L, result);
    (*L).top = (*L).top.offset(-1);
    setobj2s!(L, res, (*L).top);

    res
}
