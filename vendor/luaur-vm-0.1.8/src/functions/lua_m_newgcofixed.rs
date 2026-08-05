use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_throw_ldo::luaD_throw;
use crate::functions::newgcoblock::newgcoblock;
use crate::records::g_cheader::GCheader;
use crate::records::gc_object::GCObject;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_void;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

const K_GCO_LINK_OFFSET: usize =
    (core::mem::size_of::<GCheader>() + core::mem::size_of::<*mut c_void>() - 1)
        & !(core::mem::size_of::<*mut c_void>() - 1);

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
pub unsafe fn luaM_newgcofixed_(l: *mut lua_State, nsize: usize, memcat: u8) -> *mut GCObject {
    LUAU_ASSERT!(nsize >= K_GCO_LINK_OFFSET + core::mem::size_of::<*mut c_void>());

    let g = (*l).global;
    let nclass = sizeclass(nsize);
    LUAU_ASSERT!(nclass >= 0);

    let block = newgcoblock(l, nclass);
    if block.is_null() {
        luaD_throw(l, lua_Status::LUA_ERRMEM as i32);
    }

    (*g).totalbytes = (*g).totalbytes.wrapping_add(nsize);
    (*g).memcatbytes[memcat as usize] = (*g).memcatbytes[memcat as usize].wrapping_add(nsize);

    block as *mut GCObject
}

#[allow(unused_imports)]
pub use luaM_newgcofixed_ as lua_m_newgcofixed_;
