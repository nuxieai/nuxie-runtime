//! Node: `cxx:Function:Luau.VM:VM/src/laux.cpp:449:extendstrbuf`
//!
//! Grow a `luaL_Strbuf` past its inline buffer: allocate a GC string of the next
//! size, copy the used prefix, box it on the stack at `boxloc` (inserting a slot
//! the first time it spills off the inline buffer), and repoint p/end/storage.

use crate::functions::getnextbuffersize::getnextbuffersize;
use crate::functions::lua_insert::lua_insert;
use crate::functions::lua_pushnil::lua_pushnil;
use crate::functions::lua_s_bufstart::lua_s_bufstart;
use crate::macros::setsvalue::setsvalue;
use crate::macros::tsvalue::tsvalue;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use core::ffi::{c_char, c_int};
use luaur_common::LUAU_ASSERT;

pub fn extendstrbuf(B: *mut LuaLStrbuf, additionalsize: usize, boxloc: c_int) -> *mut c_char {
    unsafe {
        let L = (*B).L;

        if !(*B).storage.is_null() {
            LUAU_ASSERT!((*B).storage.cast_const() == tsvalue!((*L).top.offset(boxloc as isize)));
        }

        let base: *mut c_char = if !(*B).storage.is_null() {
            (*(*B).storage).data.as_mut_ptr()
        } else {
            (*B).buffer.as_mut_ptr()
        };

        let capacity = (*B).end.offset_from(base) as usize;
        let nextsize = getnextbuffersize((*B).L, capacity, capacity + additionalsize);

        let new_storage = lua_s_bufstart(L, nextsize);

        let used = (*B).p.offset_from(base) as usize;
        core::ptr::copy_nonoverlapping(base, (*new_storage).data.as_mut_ptr(), used);

        // place the string storage at the expected position in the stack
        if base == (*B).buffer.as_mut_ptr() {
            lua_pushnil(L);
            lua_insert(L, boxloc);
        }

        setsvalue!(L, (*L).top.offset(boxloc as isize), new_storage);

        (*B).p = (*new_storage).data.as_mut_ptr().add(used);
        (*B).end = (*new_storage).data.as_mut_ptr().add(nextsize);
        (*B).storage = new_storage;

        (*B).p
    }
}
