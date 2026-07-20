//! Node: `cxx:Function:Luau.VM:VM/src/lstate.cpp:130:luaE_freethread`
//! Source: `VM/src/lstate.cpp:130-138` (hand-ported)

use crate::functions::freestack::freestack;
use crate::functions::lua_m_freegco::luaM_freegco_;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn luaE_freethread(L: *mut lua_State, L1: *mut lua_State, page: *mut lua_Page) {
    let g = (*L).global;
    if let Some(userthread) = (*g).cb.userthread {
        userthread(core::ptr::null_mut(), L1);
    }

    freestack(L, L1);
    luaM_freegco_(
        L,
        L1 as *mut crate::records::gc_object::GCObject,
        core::mem::size_of::<lua_State>(),
        (*L1).hdr.memcat,
        page,
    );
}

#[allow(unused_imports)]
pub use luaE_freethread as lua_e_freethread;
