//! Node: `cxx:Function:Luau.VM:VM/src/lstate.cpp:59:f_luaopen`
//! Source: `VM/src/lstate.cpp:59-70` (hand-ported)

use crate::functions::lua_h_new::lua_h_new;
use crate::functions::lua_h_setstr::lua_h_setstr;
use crate::functions::lua_s_resize::luaS_resize;
use crate::macros::lua_s_new::luaS_new;
use crate::functions::lua_t_init::lua_t_init;
use crate::functions::stack_init::stack_init;
use crate::macros::lua_minstrtabsize::LUA_MINSTRTABSIZE;
use crate::macros::lua_s_fix::luaS_fix;
use crate::macros::lua_s_newliteral::luaS_newliteral;
use crate::macros::registry::registry;
use crate::macros::sethvalue::sethvalue;
use crate::macros::setsvalue::setsvalue;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;
use crate::type_aliases::tms::TMS;

unsafe fn weakenvalues(L: *mut lua_State, wt: *mut LuaTable) -> *mut LuaTable {
    luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauGcTraceUdata.get());
    let mt = lua_h_new(L, 0, 1);
    let slot = lua_h_setstr(L, mt, (*(*L).global).tmname[TMS::TM_MODE as usize]);
    setsvalue!(L, slot, luaS_new(L, c"v".as_ptr()));
    (*wt).metatable = mt;
    wt
}

/// open parts that may cause memory-allocation errors
pub unsafe fn f_luaopen(L: *mut lua_State, _ud: *mut core::ffi::c_void) {
    let g = (*L).global;
    stack_init(L, L); // init stack
    (*L).gt = lua_h_new(L, 0, 2); // table of globals
    sethvalue!(
        L,
        registry!(L) as *const TValue as *mut TValue,
        lua_h_new(L, 0, 2)
    ); // registry
    luaS_resize(L, LUA_MINSTRTABSIZE); // initial size of string table
    lua_t_init(L);
    if luaur_common::FFlag::LuauGcTraceUdata.get() {
        let wt = weakenvalues(L, lua_h_new(L, 0, 0));
        sethvalue!(L, core::ptr::addr_of_mut!((*g).weakregistry), wt);
    }
    luaS_fix!(luaS_newliteral(L, c"not enough memory".as_ptr())); // LUA_MEMERRMSG // pin to make sure we can always throw this error
    luaS_fix!(luaS_newliteral(L, c"error in error handling".as_ptr())); // LUA_ERRERRMSG // pin to make sure we can always throw this error
    (*g).GCthreshold = 4 * (*g).totalbytes;
}
