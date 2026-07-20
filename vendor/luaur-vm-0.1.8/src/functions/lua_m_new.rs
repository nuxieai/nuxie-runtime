use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_throw_ldo::luaD_throw;
use crate::functions::newblock::newblock;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_void;

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
pub unsafe fn luaM_new_(l: *mut lua_State, nsize: usize, memcat: u8) -> *mut c_void {
    let g = (*l).global;
    let nclass = sizeclass(nsize);

    let block = if nclass >= 0 {
        newblock(l, nclass)
    } else if let Some(frealloc) = (*g).frealloc {
        frealloc((*g).ud, core::ptr::null_mut(), 0, nsize)
    } else {
        core::ptr::null_mut()
    };

    if block.is_null() && nsize > 0 {
        luaD_throw(l, lua_Status::LUA_ERRMEM as i32);
    }

    (*g).totalbytes = (*g).totalbytes.wrapping_add(nsize);
    (*g).memcatbytes[memcat as usize] = (*g).memcatbytes[memcat as usize].wrapping_add(nsize);

    if let Some(onallocate) = (*g).cb.onallocate {
        onallocate(l, 0, nsize);
    }

    block
}

#[allow(unused_imports)]
pub use luaM_new_ as lua_m_new_;
