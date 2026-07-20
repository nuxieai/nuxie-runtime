use crate::functions::lua_s_hash::luaS_hash;
use crate::functions::lua_s_resize::luaS_resize;
use crate::macros::atom_undef::ATOM_UNDEF;
use crate::macros::lmod::lmod;
use crate::macros::whitebits::WHITEBITS;
use crate::records::gc_object::GCObject;
use crate::records::stringtable::stringtable;
use crate::records::t_string::TString;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[inline]
unsafe fn same_bytes(a: *const core::ffi::c_char, b: *const core::ffi::c_char, len: usize) -> bool {
    core::slice::from_raw_parts(a as *const u8, len)
        == core::slice::from_raw_parts(b as *const u8, len)
}

#[allow(non_snake_case)]
pub unsafe fn luaS_buffinish(l: *mut lua_State, ts: *mut TString) -> *mut TString {
    let h = luaS_hash((*ts).data.as_ptr(), (*ts).len as usize);
    let tb: *mut stringtable = core::ptr::addr_of_mut!((*(*l).global).strt);
    let bucket = lmod!(h, (*tb).size) as i32;

    let mut el = *(*tb).hash.add(bucket as usize);
    while !el.is_null() {
        if (*el).len == (*ts).len
            && same_bytes((*el).data.as_ptr(), (*ts).data.as_ptr(), (*ts).len as usize)
        {
            if crate::isdead!((*l).global, el as *mut GCObject) {
                (*el).hdr.marked ^= WHITEBITS as u8;
            }
            return el;
        }
        el = (*el).next;
    }

    LUAU_ASSERT!((*ts).next.is_null());

    (*ts).hash = h;
    *(*ts).data.as_mut_ptr().add((*ts).len as usize) = 0;
    (*ts).atom = ATOM_UNDEF as i16;
    (*ts).next = *(*tb).hash.add(bucket as usize);
    *(*tb).hash.add(bucket as usize) = ts;

    (*tb).nuse = (*tb).nuse.wrapping_add(1);
    if (*tb).nuse > (*tb).size as u32 && (*tb).size <= core::ffi::c_int::MAX / 2 {
        luaS_resize(l, (*tb).size * 2);
    }

    ts
}

#[allow(unused_imports)]
pub use luaS_buffinish as lua_s_buffinish;
