use crate::enums::lua_status::lua_Status;
use crate::enums::lua_type::lua_Type;
use crate::functions::lua_d_throw_ldo::luaD_throw;
use crate::functions::lua_g_pusherror::lua_g_pusherror;
use crate::macros::ceillog_2::ceillog2;
use crate::macros::dummynode::dummynode;
use crate::macros::lua_m_newarray::luaM_newarray;
use crate::macros::setnilvalue::setnilvalue;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

const MAXBITS: c_int = 26;

unsafe fn runerror(l: *mut lua_State, msg: *const core::ffi::c_char) -> ! {
    lua_g_pusherror(l, msg);
    luaD_throw(l, lua_Status::LUA_ERRRUN as c_int);
}

#[allow(non_snake_case)]
pub unsafe fn setnodevector(l: *mut lua_State, t: *mut LuaTable, mut size: c_int) {
    let lsize: c_int;

    if size == 0 {
        (*t).node = dummynode as *mut LuaNode;
        lsize = 0;
    } else {
        lsize = ceillog2(size as u32);
        if lsize > MAXBITS {
            runerror(l, c"table overflow".as_ptr());
        }

        size = 1 << lsize;
        (*t).node = luaM_newarray!(l, size as usize, LuaNode, (*t).memcat);

        let mut i = 0;
        while i < size {
            let n = (*t).node.add(i as usize);
            (*n).key.set_next(0);
            (*n).key.value = Default::default();
            (*n).key.extra = [0];
            (*n).key.set_tt(lua_Type::LUA_TNIL as i32);
            setnilvalue!(core::ptr::addr_of_mut!((*n).val));
            i += 1;
        }
    }

    (*t).lsizenode = lsize as u8;
    (*t).nodemask8 = ((1 << lsize) - 1) as u8;
    (*t).union.lastfree = size;
}
