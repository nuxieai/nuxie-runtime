use crate::macros::lua_d_checkstack::luaD_checkstack;
use crate::macros::setobj_2_s::setobj2s;

use crate::functions::lua_d_call::lua_d_call;

use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::t_value::TValue;

use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub unsafe fn call_tm(
    L: *mut LuaState,
    f: *const TValue,
    p1: *const TValue,
    p2: *const TValue,
    p3: *const TValue,
) {
    LUAU_ASSERT!((*L).top.offset(4) < (*L).stack.add((*L).stacksize as usize));

    setobj2s!(L, (*L).top, f);
    setobj2s!(L, (*L).top.add(1), p1);
    setobj2s!(L, (*L).top.add(2), p2);
    setobj2s!(L, (*L).top.add(3), p3);

    luaD_checkstack!(L, 4);
    (*L).top = (*L).top.add(4);

    lua_d_call(L, (*L).top.offset(-4), 0);
}
