use crate::functions::lua_m_free::luaM_free_;
use crate::macros::lua_m_freearray::luaM_freearray;
use crate::type_aliases::call_info::CallInfo;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn freestack(L: *mut lua_State, L1: *mut lua_State) {
    luaM_freearray!(
        L,
        (*L1).base_ci,
        (*L1).size_ci as usize,
        CallInfo,
        (*L1).activememcat
    );
    luaM_freearray!(
        L,
        (*L1).stack,
        (*L1).stacksize as usize,
        TValue,
        (*L1).activememcat
    );
}
