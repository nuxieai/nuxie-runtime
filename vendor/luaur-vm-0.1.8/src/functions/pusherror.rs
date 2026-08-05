use crate::functions::currentline::currentline;
use crate::functions::getluaproto::get_lua_proto;
use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::macros::getstr::getstr;
use crate::macros::is_lua::isLua;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, CStr};

unsafe fn push_error_bytes(
    L: *mut lua_State,
    source: Option<*const c_char>,
    line: i32,
    msg: *const c_char,
) {
    let msg = CStr::from_ptr(msg).to_bytes();
    let mut result = alloc::vec::Vec::new();

    if let Some(source) = source {
        let source = CStr::from_ptr(source).to_bytes();
        let line = alloc::format!("{line}");
        result.reserve(source.len() + line.len() + msg.len() + 3);
        result.extend_from_slice(source);
        result.push(b':');
        result.extend_from_slice(line.as_bytes());
        result.extend_from_slice(b": ");
    } else {
        result.reserve(msg.len() + 3);
        result.extend_from_slice(b":: ");
    }

    result.extend_from_slice(msg);
    lua_pushlstring(L, result.as_ptr().cast(), result.len());
}

#[export_name = "luaur_pusherror"]
pub unsafe fn pusherror(L: *mut lua_State, msg: *const c_char) {
    let ci = (*L).ci;

    // isLua! macro expects a pointer to CallInfo, not a dereferenced struct.
    if isLua!(ci) {
        let proto = get_lua_proto(ci);
        let source = (*proto).source;
        let line = currentline(L, ci);
        push_error_bytes(L, Some(getstr(source)), line, msg);
    } else {
        push_error_bytes(L, None, 0, msg);
    }
}
