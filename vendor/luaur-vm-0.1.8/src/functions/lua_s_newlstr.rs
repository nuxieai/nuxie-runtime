use crate::functions::lua_s_hash::luaS_hash;
use crate::functions::newlstr::newlstr;
use crate::macros::getstr::getstr;
use crate::macros::lmod::lmod;
use crate::macros::whitebits::WHITEBITS;
use crate::records::gc_object::GCObject;
use crate::records::t_string::TString;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

#[inline]
unsafe fn same_bytes(a: *const c_char, b: *const c_char, len: usize) -> bool {
    core::slice::from_raw_parts(a as *const u8, len)
        == core::slice::from_raw_parts(b as *const u8, len)
}

#[allow(non_snake_case)]
pub unsafe fn luaS_newlstr(l: *mut lua_State, str_: *const c_char, len: usize) -> *mut TString {
    let h = luaS_hash(str_, len);
    let bucket = lmod!(h, (*(*l).global).strt.size);
    let mut el = *(*(*l).global).strt.hash.add(bucket as usize);

    while !el.is_null() {
        if (*el).len as usize == len && same_bytes(str_, getstr(el), len) {
            if crate::isdead!((*l).global, el as *mut GCObject) {
                (*el).hdr.marked ^= WHITEBITS as u8;
            }
            return el;
        }
        el = (*el).next;
    }

    newlstr(l, str_, len, h)
}

#[allow(unused_imports)]
pub use luaS_newlstr as lua_s_newlstr;
