//! Node: `cxx:Function:Luau.VM:VM/src/lstate.cpp:92:close_state`
//! Source: `VM/src/lstate.cpp:92-113` (hand-ported)

use crate::functions::freestack::freestack;
use crate::functions::lua_c_freeall::luaC_freeall;
use crate::functions::lua_f_close::lua_f_close;
use crate::functions::lua_m_free::luaM_free_;
use crate::macros::lua_memory_categories::LUA_MEMORY_CATEGORIES;
use crate::macros::lua_sizeclasses::LUA_SIZECLASSES;
use crate::records::lg::LG;
use crate::records::t_string::TString;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub unsafe fn close_state(L: *mut lua_State) {
    let g = (*L).global;
    lua_f_close(L, (*L).stack); // close all upvalues for this thread
    luaC_freeall(L); // collect all objects
    LUAU_ASSERT!((*g).strt.nuse == 0);
    luaM_free_(
        L,
        (*(*L).global).strt.hash as *mut core::ffi::c_void,
        (*(*L).global).strt.size as usize * core::mem::size_of::<*mut TString>(),
        0,
    );
    freestack(L, L);
    for i in 0..LUA_SIZECLASSES as usize {
        LUAU_ASSERT!((*g).freepages[i].is_null());
        LUAU_ASSERT!((*g).freegcopages[i].is_null());
    }
    LUAU_ASSERT!((*g).allgcopages.is_null());
    LUAU_ASSERT!((*g).totalbytes == core::mem::size_of::<LG>());
    LUAU_ASSERT!((*g).memcatbytes[0] == core::mem::size_of::<LG>());
    for i in 1..LUA_MEMORY_CATEGORIES as usize {
        LUAU_ASSERT!((*g).memcatbytes[i] == 0);
    }

    if let Some(close) = (*(*L).global).ecb.close {
        close(L);
    }

    if let Some(frealloc) = (*g).frealloc {
        frealloc(
            (*g).ud,
            L as *mut core::ffi::c_void,
            core::mem::size_of::<LG>(),
            0,
        );
    }
}
