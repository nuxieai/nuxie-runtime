use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::functions::lua_s_buffinish::lua_s_buffinish;
use crate::functions::lua_s_newlstr::luaS_newlstr;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::setsvalue::setsvalue;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use crate::records::t_string::TString;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_l_pushresult(B: *mut LuaLStrbuf) {
    let L = (*B).L;
    let storage = (*B).storage;

    if !storage.is_null() {
        luaC_checkGC!(L);

        let p = (*B).p;
        let end = (*B).end;

        if p == end {
            setsvalue!(L, (*L).top.offset(-1), lua_s_buffinish(L, storage));
        } else {
            let storage_data = (*storage).data.as_ptr();
            let len = (p as usize).wrapping_sub(storage_data as usize);
            setsvalue!(L, (*L).top.offset(-1), luaS_newlstr(L, storage_data, len));
        }
    } else {
        let p = (*B).p;
        let buffer = (*B).buffer.as_ptr();
        let len = (p as usize).wrapping_sub(buffer as usize);
        lua_pushlstring(L, buffer, len);
    }
}
