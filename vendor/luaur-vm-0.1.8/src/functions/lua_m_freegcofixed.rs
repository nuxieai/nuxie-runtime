use crate::enums::lua_type::lua_Type;
use crate::functions::freegcoblock::freegcoblock;
use crate::records::gc_object::GCObject;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_void;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[inline]
fn sizeclass(size: usize) -> i32 {
    if size == 0 || size > 1024 {
        -1
    } else if size <= 56 {
        ((size + 7) / 8 - 1) as i32
    } else if size <= 240 {
        (7 + (size - 49) / 16) as i32
    } else if size <= 480 {
        (19 + (size - 225) / 32) as i32
    } else {
        (27 + (size - 449) / 64) as i32
    }
}

#[allow(non_snake_case)]
pub unsafe fn luaM_freegcofixed_(
    l: *mut lua_State,
    block: *mut GCObject,
    osize: usize,
    memcat: u8,
    page: *mut lua_Page,
) {
    let g = (*l).global;
    let oclass = sizeclass(osize);
    LUAU_ASSERT!(oclass >= 0);

    (*block).gch.tt = lua_Type::LUA_TNIL as u8;
    freegcoblock(l, oclass, block as *mut c_void, page);

    (*g).totalbytes = (*g).totalbytes.wrapping_sub(osize);
    (*g).memcatbytes[memcat as usize] = (*g).memcatbytes[memcat as usize].wrapping_sub(osize);
}

#[allow(unused_imports)]
pub use luaM_freegcofixed_ as lua_m_freegcofixed_;
