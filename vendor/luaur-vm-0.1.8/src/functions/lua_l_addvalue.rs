use crate::functions::extendstrbuf::extendstrbuf;
use crate::functions::lua_tolstring::lua_tolstring;
use crate::macros::lua_pop::lua_pop;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;
use core::ptr::copy_nonoverlapping;

pub fn lua_l_addvalue(B: *mut LuaLStrbuf) {
    unsafe {
        let L = (*B).L;

        let mut vl: usize = 0;
        let s = lua_tolstring(L, -1, &mut vl);

        if !s.is_null() {
            let current_buffer_size = (*B).end as usize - (*B).p as usize;
            if current_buffer_size < vl as usize {
                extendstrbuf(B, vl as usize - current_buffer_size, -2);
            }

            copy_nonoverlapping(s as *const u8, (*B).p as *mut u8, vl as usize);
            (*B).p = (*B).p.add(vl as usize);

            lua_pop(L, 1);
        }
    }
}
