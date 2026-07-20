//! Node: `cxx:Function:Luau.VM:VM/src/lstate.cpp:116:luaE_newthread`
//! Source: `VM/src/lstate.cpp:116-128` (hand-ported)

use crate::enums::lua_type::lua_Type;
use crate::functions::lua_m_newgco::luaM_newgco_;
use crate::functions::preinit_state::preinit_state;
use crate::functions::stack_init::stack_init;
use crate::macros::iswhite::iswhite;
use crate::macros::lua_c_init::luaC_init;
use crate::records::gc_object::GcObject;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn luaE_newthread(L: *mut lua_State) -> *mut lua_State {
    let L1 =
        luaM_newgco_(L, core::mem::size_of::<lua_State>(), (*L).activememcat) as *mut lua_State;
    luaC_init!(L, L1, lua_Type::LUA_TTHREAD as i32);
    preinit_state(L1, (*L).global);
    (*L1).activememcat = (*L).activememcat; // inherit the active memory category
    stack_init(L1, L); // init stack
    (*L1).gt = (*L).gt; // share table of globals
    (*L1).singlestep = (*L).singlestep;
    LUAU_ASSERT!(iswhite!(L1 as *mut GcObject)); // iswhite(obj2gco(L1))
    L1
}

#[allow(unused_imports)]
pub use luaE_newthread as lua_e_newthread;
