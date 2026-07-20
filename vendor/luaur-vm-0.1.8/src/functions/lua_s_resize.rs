use crate::macros::lmod::lmod;
use crate::macros::lua_m_freearray::luaM_freearray;
use crate::macros::lua_m_newarray::luaM_newarray;
use crate::records::stringtable::stringtable;
use crate::records::t_string::TString;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn luaS_resize(l: *mut lua_State, newsize: c_int) {
    let newhash = luaM_newarray!(l, newsize as usize, *mut TString, 0);
    let tb: *mut stringtable = core::ptr::addr_of_mut!((*(*l).global).strt);

    let mut i = 0;
    while i < newsize {
        *newhash.add(i as usize) = core::ptr::null_mut();
        i += 1;
    }

    i = 0;
    while i < (*tb).size {
        let mut p = *(*tb).hash.add(i as usize);
        while !p.is_null() {
            let next = (*p).next;
            let h = (*p).hash;
            let h1 = lmod!(h, newsize) as c_int;
            LUAU_ASSERT!((h % newsize as u32) as c_int == lmod!(h, newsize));
            (*p).next = *newhash.add(h1 as usize);
            *newhash.add(h1 as usize) = p;
            p = next;
        }
        i += 1;
    }

    luaM_freearray!(l, (*tb).hash, (*tb).size as usize, *mut TString, 0);
    (*tb).size = newsize;
    (*tb).hash = newhash;
}

#[allow(unused_imports)]
pub use luaS_resize as lua_s_resize;
