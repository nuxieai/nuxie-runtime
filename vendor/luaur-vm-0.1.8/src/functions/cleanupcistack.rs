use crate::macros::clvalue::clvalue;
use crate::records::call_info::CallInfo;
use crate::records::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn cleanupcistack(L: *mut lua_State) {
    let mut lastci: *mut CallInfo = (*L).ci;
    while lastci != (*L).base_ci {
        let func = (*lastci).func;
        let closure = clvalue!(func) as *const _ as *mut crate::records::closure::Closure;
        LUAU_ASSERT!((*closure).usage > 0);
        (*closure).usage -= 1;
        lastci = lastci.offset(-1);
    }
}
