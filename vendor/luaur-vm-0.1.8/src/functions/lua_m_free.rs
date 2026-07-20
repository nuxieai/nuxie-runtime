use crate::functions::freeblock::freeblock;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_void;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn luaM_free_(l: *mut lua_State, block: *mut c_void, osize: usize, memcat: u8) {
    let g = (*l).global;
    LUAU_ASSERT!((osize == 0) == block.is_null());

    let sizeclass = |size: usize| -> i32 {
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
    };

    let oclass = sizeclass(osize);

    if oclass >= 0 {
        freeblock(l, oclass, block);
    } else if let Some(frealloc) = (*g).frealloc {
        frealloc((*g).ud, block, osize, 0);
    }

    (*g).totalbytes = (*g).totalbytes.wrapping_sub(osize);
    let memcatbytes = core::ptr::addr_of_mut!((*g).memcatbytes) as *mut usize;
    *memcatbytes.add(memcat as usize) = (*memcatbytes.add(memcat as usize)).wrapping_sub(osize);
}
