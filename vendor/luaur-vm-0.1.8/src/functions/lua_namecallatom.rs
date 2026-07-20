use crate::macros::atom_undef::ATOM_UNDEF;
use crate::macros::getstr::getstr;
use crate::macros::lua_s_updateatom::luaS_updateatom;
use crate::records::lua_state::lua_State;
use crate::records::t_string::TString;
use crate::type_aliases::lua_state::lua_State as lua_State_alias;
use crate::type_aliases::t_string::TString as TString_alias;

#[allow(non_snake_case)]
pub unsafe fn lua_namecallatom(
    L: *mut lua_State_alias,
    atom: *mut i32,
) -> *const core::ffi::c_char {
    let s = (*L).namecall;
    if s.is_null() {
        return core::ptr::null();
    }
    if !atom.is_null() {
        luaS_updateatom!(L, s);
        *atom = (*s).atom as i32;
    }
    getstr(s)
}
