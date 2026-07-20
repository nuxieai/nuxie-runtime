use crate::enums::lua_type::lua_Type;
use crate::macros::lua_c_init::luaC_init;
use crate::macros::lua_minstack::LUA_MINSTACK;
use crate::macros::size_cclosure::size_cclosure;
use crate::records::closure::{CClosure, Closure};
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn luaF_newCclosure(l: *mut lua_State, nelems: c_int, e: *mut LuaTable) -> *mut Closure {
    let c =
        crate::functions::lua_m_newgco::luaM_newgco_(l, size_cclosure(nelems), (*l).activememcat)
            as *mut Closure;

    luaC_init!(l, c, lua_Type::LUA_TFUNCTION as c_int);
    (*c).isC = 1;
    (*c).env = e;
    (*c).nupvalues = nelems as u8;
    (*c).stacksize = LUA_MINSTACK as u8;
    (*c).preload = 0;
    (*c).usage = 0;
    (*c).gclist = core::ptr::null_mut();
    let cc = core::ptr::addr_of_mut!((*c).inner.c) as *mut CClosure;
    (*cc).f = None;
    (*cc).cont = None;
    (*cc).debugname = core::ptr::null();

    c
}

#[allow(unused_imports)]
pub use luaF_newCclosure as lua_f_new_cclosure;
