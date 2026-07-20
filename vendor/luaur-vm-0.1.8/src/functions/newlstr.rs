use crate::enums::lua_type::lua_Type;
use crate::functions::lua_m_toobig::lua_m_toobig;
use crate::functions::lua_s_resize::luaS_resize;
use crate::macros::atom_undef::ATOM_UNDEF;
use crate::macros::lmod::lmod;
use crate::macros::lua_c_init::luaC_init;
use crate::macros::maxssize::MAXSSIZE;
use crate::macros::sizestring::sizestring;
use crate::records::stringtable::stringtable;
use crate::records::t_string::TString;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int, c_uint};

#[allow(non_snake_case)]
pub unsafe fn newlstr(
    l: *mut lua_State,
    str_: *const c_char,
    len: usize,
    mut h: c_uint,
) -> *mut TString {
    if len > MAXSSIZE as usize {
        lua_m_toobig(l);
    }

    let ts = crate::functions::lua_m_newgco::luaM_newgco_(l, sizestring(len), (*l).activememcat)
        as *mut TString;

    luaC_init!(l, ts, lua_Type::LUA_TSTRING as c_int);
    (*ts).atom = ATOM_UNDEF as i16;
    (*ts).hash = h;
    (*ts).len = len as c_uint;

    core::ptr::copy_nonoverlapping(str_, (*ts).data.as_mut_ptr(), len);
    *(*ts).data.as_mut_ptr().add(len) = 0;

    let tb: *mut stringtable = core::ptr::addr_of_mut!((*(*l).global).strt);
    h = lmod!(h, (*tb).size) as c_uint;
    (*ts).next = *(*tb).hash.add(h as usize);
    *(*tb).hash.add(h as usize) = ts;

    (*tb).nuse = (*tb).nuse.wrapping_add(1);
    if (*tb).nuse > (*tb).size as u32 && (*tb).size <= c_int::MAX / 2 {
        luaS_resize(l, (*tb).size * 2);
    }

    ts
}
