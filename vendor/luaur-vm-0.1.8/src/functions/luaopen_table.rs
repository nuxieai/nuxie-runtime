//! Node: `cxx:Function:Luau.VM:VM/src/ltablib.cpp:619:luaopen_table`
//! Source: `VM/src/ltablib.cpp:596-628` (hand-ported)

use crate::functions::foreach::foreach;
use crate::functions::foreachi::foreachi;
use crate::functions::getn::getn;
use crate::functions::lua_l_register::lua_l_register;
use crate::functions::lua_setfield::lua_setfield;
use crate::functions::maxn::maxn;
use crate::functions::tclear::tclear;
use crate::functions::tclone::tclone;
use crate::functions::tconcat::tconcat;
use crate::functions::tcreate::tcreate;
use crate::functions::tfind::tfind;
use crate::functions::tfreeze::tfreeze;
use crate::functions::tinsert::tinsert;
use crate::functions::tisfrozen::tisfrozen;
use crate::functions::tmove::tmove;
use crate::functions::tpack::tpack;
use crate::functions::tremove::tremove;
use crate::functions::tsort::tsort;
use crate::functions::tunpack::tunpack;
use crate::macros::lua_pushcfunction::LUA_PUSHCFUNCTION;
use crate::macros::lua_setglobal::lua_setglobal;
use crate::records::lua_l_reg::LuaLReg;
use crate::type_aliases::lua_state::lua_State;

struct TabFuncs([LuaLReg; 18]);
unsafe impl Sync for TabFuncs {}

static TAB_FUNCS: TabFuncs = TabFuncs([
    LuaLReg {
        name: c"concat".as_ptr(),
        func: Some(tconcat),
    },
    LuaLReg {
        name: c"foreach".as_ptr(),
        func: Some(foreach),
    },
    LuaLReg {
        name: c"foreachi".as_ptr(),
        func: Some(foreachi),
    },
    LuaLReg {
        name: c"getn".as_ptr(),
        func: Some(getn),
    },
    LuaLReg {
        name: c"maxn".as_ptr(),
        func: Some(maxn),
    },
    LuaLReg {
        name: c"insert".as_ptr(),
        func: Some(tinsert),
    },
    LuaLReg {
        name: c"remove".as_ptr(),
        func: Some(tremove),
    },
    LuaLReg {
        name: c"sort".as_ptr(),
        func: Some(tsort),
    },
    LuaLReg {
        name: c"pack".as_ptr(),
        func: Some(tpack),
    },
    LuaLReg {
        name: c"unpack".as_ptr(),
        func: Some(tunpack),
    },
    LuaLReg {
        name: c"move".as_ptr(),
        func: Some(tmove),
    },
    LuaLReg {
        name: c"create".as_ptr(),
        func: Some(tcreate),
    },
    LuaLReg {
        name: c"find".as_ptr(),
        func: Some(tfind),
    },
    LuaLReg {
        name: c"clear".as_ptr(),
        func: Some(tclear),
    },
    LuaLReg {
        name: c"freeze".as_ptr(),
        func: Some(tfreeze),
    },
    LuaLReg {
        name: c"isfrozen".as_ptr(),
        func: Some(tisfrozen),
    },
    LuaLReg {
        name: c"clone".as_ptr(),
        func: Some(tclone),
    },
    LuaLReg {
        name: core::ptr::null(),
        func: None,
    },
]);

#[allow(non_snake_case)]
pub unsafe fn luaopen_table(L: *mut lua_State) -> core::ffi::c_int {
    lua_l_register(L, c"table".as_ptr(), TAB_FUNCS.0.as_ptr());

    LUA_PUSHCFUNCTION(L, Some(tunpack), c"unpack".as_ptr());
    lua_setglobal(L, c"unpack".as_ptr());

    1
}
