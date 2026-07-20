use crate::functions::getthread::getthread;
use crate::functions::lua_getinfo::lua_getinfo;
use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_isnumber::lua_isnumber;
use crate::functions::lua_pushboolean::lua_pushboolean;
use crate::functions::lua_pushinteger::lua_pushinteger;
use crate::functions::lua_pushstring::lua_pushstring;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::lua_rawcheckstack::lua_rawcheckstack;
use crate::functions::lua_settop::lua_settop;
use crate::functions::lua_xmove::lua_xmove;
use crate::macros::lua_isfunction::lua_isfunction;
use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::macros::lua_l_argerror::luaL_argerror;
use crate::macros::lua_l_checkstring::luaL_checkstring;
use crate::macros::lua_tointeger::lua_tointeger;
use crate::records::lua_debug::LuaDebug;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn db_info(L: *mut lua_State) -> core::ffi::c_int {
    let mut arg: core::ffi::c_int = 0;
    let L1 = getthread(L, &mut arg);
    let mut l1top: core::ffi::c_int = 0;

    // if L1 != L, L1 can be in any state, and therefore there are no guarantees about its stack space
    if L != L1 {
        // for 'f' option, we reserve one slot and we also record the stack top
        lua_rawcheckstack(L1, 1);
        l1top = lua_gettop(L1);
    }

    let level: core::ffi::c_int;
    if lua_isnumber(L, arg + 1) != 0 {
        level = lua_tointeger!(L, arg + 1);
        luaL_argcheck!(L, level >= 0, arg + 1, "level can't be negative");
    } else if arg == 0 && lua_isfunction!(L, 1) {
        // convert absolute index to relative index
        level = -lua_gettop(L);
    } else {
        luaL_argerror!(L, arg + 1, "function or level expected");
    }

    let options = luaL_checkstring!(L, arg + 2);

    let mut ar: LuaDebug = core::mem::zeroed();
    if lua_getinfo(L1, level, options, &mut ar) == 0 {
        return 0;
    }

    let mut results: core::ffi::c_int = 0;
    let mut occurs = [false; 26];

    let mut it = options;
    while *it != 0 {
        let ch = *it as u8;
        if ch >= b'a' && ch <= b'z' {
            let idx = (ch - b'a') as usize;
            if occurs[idx] {
                // restore stack state of another thread as 'f' option might not have been visited yet
                if L != L1 {
                    lua_settop(L1, l1top);
                }

                luaL_argerror!(L, arg + 2, "duplicate option");
            }
            occurs[idx] = true;
        }

        match ch {
            b's' => {
                lua_pushstring(L, ar.short_src);
                results += 1;
            }
            b'l' => {
                lua_pushinteger(L, ar.currentline);
                results += 1;
            }
            b'n' => {
                lua_pushstring(
                    L,
                    if !ar.name.is_null() {
                        ar.name
                    } else {
                        c"".as_ptr()
                    },
                );
                results += 1;
            }
            b'f' => {
                if L1 == L {
                    lua_pushvalue(L, -1 - results); // function is right before results
                } else {
                    lua_xmove(L1, L, 1); // function is at top of L1
                }
                results += 1;
            }
            b'a' => {
                lua_pushinteger(L, ar.nparams as core::ffi::c_int);
                lua_pushboolean(L, ar.isvararg as core::ffi::c_int);
                results += 2;
            }
            _ => {
                // restore stack state of another thread as 'f' option might not have been visited yet
                if L != L1 {
                    lua_settop(L1, l1top);
                }

                luaL_argerror!(L, arg + 2, "invalid option");
            }
        }

        it = it.add(1);
    }

    results
}
