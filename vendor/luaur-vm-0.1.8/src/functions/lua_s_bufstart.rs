use crate::functions::lua_m_toobig::lua_m_toobig;
use crate::macros::atom_undef::ATOM_UNDEF;
use crate::macros::lua_c_init::luaC_init;
use crate::macros::maxssize::MAXSSIZE;
use crate::macros::sizestring::sizestring;
use crate::records::t_string::TString;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn luaS_bufstart(l: *mut lua_State, size: usize) -> *mut TString {
    if size > MAXSSIZE as usize {
        lua_m_toobig(l);
    }

    let ts = crate::functions::lua_m_newgco::luaM_newgco_(l, sizestring(size), (*l).activememcat)
        as *mut TString;

    luaC_init!(
        l,
        ts,
        crate::enums::lua_type::lua_Type::LUA_TSTRING as c_int
    );
    (*ts).atom = ATOM_UNDEF as i16;
    (*ts).hash = 0;
    (*ts).len = size as u32;
    (*ts).next = core::ptr::null_mut();

    ts
}

#[allow(unused_imports)]
pub use luaS_bufstart as lua_s_bufstart;
