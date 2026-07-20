use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::macros::api_check::api_check;
use crate::macros::api_checknelems::api_checknelems;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_c_barrier::luaC_barrier;
use crate::macros::lua_environindex::LUA_ENVIRONINDEX;
use crate::macros::lua_globalsindex::LUA_GLOBALSINDEX;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::setobj::setobj;
use crate::macros::ttistable::ttistable;
use crate::records::lua_state::lua_State;
use crate::type_aliases::closure::Closure;
use crate::type_aliases::stk_id::StkId;

unsafe fn current_closure(L: *mut lua_State) -> *mut Closure {
    let func = (*(*L).ci).func;
    core::ptr::addr_of_mut!((*(*func).value.gc).cl) as *mut Closure
}

#[allow(non_snake_case)]
pub unsafe fn lua_replace(L: *mut lua_State, idx: core::ffi::c_int) {
    api_checknelems!(L, 1);
    lua_c_threadbarrier_lapi(L);
    let o: StkId = index2addr(L, idx);
    api_check!(L, o != luaO_nilobject as StkId);
    if idx == LUA_ENVIRONINDEX {
        api_check!(L, (*L).ci != (*L).base_ci);
        let func: *mut Closure = current_closure(L);
        api_check!(L, ttistable!((*L).top.offset(-1)));
        (*func).env = hvalue!((*L).top.offset(-1));
        luaC_barrier!(L, func, (*L).top.offset(-1));
    } else if idx == LUA_GLOBALSINDEX {
        api_check!(L, ttistable!((*L).top.offset(-1)));
        (*L).gt = hvalue!((*L).top.offset(-1));
    } else {
        setobj!(L, o, (*L).top.offset(-1));
        if idx < LUA_GLOBALSINDEX {
            luaC_barrier!(L, current_closure(L), (*L).top.offset(-1));
        }
    }
    (*L).top = (*L).top.offset(-1);
}
