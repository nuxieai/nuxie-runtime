use crate::functions::lua_l_pushresult::lua_l_pushresult;
use crate::records::lua_l_strbuf::LuaLStrbuf;

#[allow(non_snake_case)]
pub fn lua_l_pushresultsize(B: *mut LuaLStrbuf, size: usize) {
    unsafe {
        (*B).p = (*B).p.wrapping_add(size);
        lua_l_pushresult(B);
    }
}
