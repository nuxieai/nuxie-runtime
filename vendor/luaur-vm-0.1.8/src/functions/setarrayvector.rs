use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_throw_ldo::luaD_throw;
use crate::functions::lua_g_pusherror::lua_g_pusherror;
use crate::functions::lua_m_realloc::lua_m_realloc_;
use crate::macros::setnilvalue::setnilvalue;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;
use core::ffi::c_int;

const MAXSIZE: c_int = 1 << 26;

unsafe fn runerror(l: *mut lua_State, msg: *const core::ffi::c_char) -> ! {
    lua_g_pusherror(l, msg);
    luaD_throw(l, lua_Status::LUA_ERRRUN as c_int);
}

#[allow(non_snake_case)]
pub unsafe fn setarrayvector(l: *mut lua_State, t: *mut LuaTable, size: c_int) {
    if size > MAXSIZE {
        runerror(l, c"table overflow".as_ptr());
    }

    let oldsize = (*t).sizearray;
    let newarray = lua_m_realloc_(
        l,
        (*t).array as *mut core::ffi::c_void,
        oldsize as usize * core::mem::size_of::<TValue>(),
        size as usize * core::mem::size_of::<TValue>(),
        (*t).memcat,
    ) as *mut TValue;

    (*t).array = newarray;

    let mut i = oldsize;
    while i < size {
        setnilvalue!((*t).array.add(i as usize));
        i += 1;
    }

    (*t).sizearray = size;
}
