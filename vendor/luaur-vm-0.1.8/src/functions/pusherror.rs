use crate::functions::currentline::currentline;
use crate::functions::getluaproto::get_lua_proto;
use crate::functions::lua_o_pushfstring::luaO_pushfstring;
use crate::macros::getstr::getstr;
use crate::macros::is_lua::isLua;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, CStr};

#[export_name = "luaur_pusherror"]
pub unsafe fn pusherror(L: *mut lua_State, msg: *const c_char) {
    let ci = (*L).ci;

    // isLua! macro expects a pointer to CallInfo, not a dereferenced struct.
    if isLua!(ci) {
        let proto = get_lua_proto(ci);
        let source = (*proto).source;
        let line = currentline(L, ci);
        let chunk = CStr::from_ptr(getstr(source)).to_string_lossy();
        let msg_str = CStr::from_ptr(msg).to_string_lossy();
        luaO_pushfstring(
            L,
            c"%s:%d: %s".as_ptr(),
            format_args!("{}:{}: {}", chunk, line, msg_str),
        );
    } else {
        let msg_str = CStr::from_ptr(msg).to_string_lossy();
        luaO_pushfstring(L, c":: %s".as_ptr(), format_args!(":: {}", msg_str));
    }
}
