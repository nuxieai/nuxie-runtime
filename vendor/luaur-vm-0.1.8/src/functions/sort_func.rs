use crate::functions::lua_d_call::lua_d_call;
use crate::macros::l_isfalse::l_isfalse;
use crate::macros::setobj_2_s::setobj_2_s;
use crate::records::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub unsafe fn sort_func(L: *mut lua_State, l: *const TValue, r: *const TValue) -> core::ffi::c_int {
    LUAU_ASSERT!(unsafe { (*L).top == (*L).base.offset(2) }); // table, function

    let top = unsafe { (*L).top };
    let base = unsafe { (*L).base };

    setobj_2_s!(L, top, base.offset(1));
    setobj_2_s!(L, top.offset(1), l);
    setobj_2_s!(L, top.offset(2), r);

    unsafe {
        (*L).top = top.offset(3); // safe because of LUA_MINSTACK guarantee
        lua_d_call(L, top, 1);
        (*L).top = (*L).top.offset(-1); // maintain stack depth

        (!l_isfalse!((*L).top)) as core::ffi::c_int
    }
}
