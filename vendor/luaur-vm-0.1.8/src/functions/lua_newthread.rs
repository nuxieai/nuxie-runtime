use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_e_newthread::lua_e_newthread;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::setthvalue::setthvalue;
use crate::records::global_state::global_State;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn lua_newthread(L: *mut lua_State) -> *mut lua_State {
    luaC_checkGC!(L);
    lua_c_threadbarrier_lapi(L);
    let L1 = lua_e_newthread(L);
    setthvalue!(L, (*L).top, L1);
    api_incr_top!(L);
    let g = (*L).global as *mut global_State;
    if let Some(userthread) = (*g).cb.userthread {
        userthread(L, L1);
    }
    L1
}
