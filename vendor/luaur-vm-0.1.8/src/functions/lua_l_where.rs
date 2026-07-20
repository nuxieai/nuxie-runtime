//! Node: `cxx:Function:Luau.VM:VM/src/laux.cpp:71:luaL_where`
//! Source: `VM/src/laux.cpp:71-83` (hand-ported)

use core::ffi::c_int;

use crate::functions::currentline::currentline;
use crate::functions::getluaproto::get_lua_proto;
use crate::functions::lua_o_chunkid::lua_o_chunkid;
use crate::functions::lua_o_pushfstring::luaO_pushfstring;
use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::functions::lua_rawcheckstack::lua_rawcheckstack;
use crate::macros::getstr::getstr;
use crate::macros::is_lua::isLua;
use crate::macros::lua_idsize::LUA_IDSIZE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_l_where(L: *mut lua_State, level: c_int) {
    let mut ci = (*L).ci;
    for _ in 0..level {
        if ci == (*L).base_ci {
            lua_rawcheckstack(L, 1);
            lua_pushlstring(L, c"".as_ptr(), 0);
            return;
        }
        ci = ci.sub(1);
    }

    if isLua!(ci) {
        let proto = get_lua_proto(ci);
        let source = (*proto).source;
        let mut chunkbuf = [0; LUA_IDSIZE as usize];
        let chunkid = lua_o_chunkid(
            chunkbuf.as_mut_ptr(),
            chunkbuf.len(),
            getstr(source),
            (*source).len as usize,
        );
        let line = currentline(L, ci);
        if line > 0 {
            let chunk = core::ffi::CStr::from_ptr(chunkid).to_string_lossy();
            luaO_pushfstring(L, c"%s:%d: ".as_ptr(), format_args!("{}:{}: ", chunk, line));
            return;
        }
    }

    lua_rawcheckstack(L, 1);
    lua_pushlstring(L, c"".as_ptr(), 0);
}
