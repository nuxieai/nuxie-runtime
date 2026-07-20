//! Node: `cxx:Function:Luau.VM:VM/src/lstate.cpp:292:lua_close`
//! Source: `VM/src/lstate.cpp:292-297` (hand-ported)

use crate::functions::close_state::close_state;
use crate::functions::lua_f_close::lua_f_close;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn lua_close(L: *mut lua_State) {
    let L = (*(*L).global).mainthread; // only the main thread can be closed
    lua_f_close(L, (*L).stack); // close all upvalues for this thread
    close_state(L);
}
