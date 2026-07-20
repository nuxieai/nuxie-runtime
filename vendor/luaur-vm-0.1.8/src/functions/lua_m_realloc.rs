//! Node: `cxx:Function:Luau.VM:VM/src/lmem.cpp:599:luaM_realloc_`
//! Source: `VM/src/lmem.cpp:599-640` (hand-fixed: the translated version
//! invented a power-of-2 size-class scheme that disagreed with the real
//! progressive table in `sizeclass!`, so realloc'd blocks were freed under
//! the wrong class and tripped the blockSize assert in `freeblock`)

use crate::enums::lua_status::lua_Status;
use crate::functions::freeblock::freeblock;
use crate::functions::lua_d_throw_ldo::lua_d_throw;
use crate::functions::newblock::newblock;
use crate::macros::sizeclass::sizeclass;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_void;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_unlikely::LUAU_UNLIKELY;

#[allow(non_snake_case)]
pub unsafe fn lua_m_realloc_(
    l: *mut lua_State,
    block: *mut c_void,
    osize: usize,
    nsize: usize,
    memcat: u8,
) -> *mut c_void {
    let g = (*l).global;
    LUAU_ASSERT!((osize == 0) == (block.is_null()));

    let nclass = sizeclass!(nsize) as i32;
    let oclass = sizeclass!(osize) as i32;
    let result: *mut c_void;

    // if either block needs to be allocated using a block allocator, we can't use realloc directly
    if nclass >= 0 || oclass >= 0 {
        result = if nclass >= 0 {
            newblock(l, nclass)
        } else {
            ((*g).frealloc.expect("frealloc is null"))((*g).ud, core::ptr::null_mut(), 0, nsize)
        };

        if result.is_null() && nsize > 0 {
            lua_d_throw(l, lua_Status::LUA_ERRMEM as i32);
        }

        if osize > 0 && nsize > 0 {
            let copy_size = if osize < nsize { osize } else { nsize };
            core::ptr::copy_nonoverlapping(block as *const u8, result as *mut u8, copy_size);
        }

        if oclass >= 0 {
            freeblock(l, oclass, block);
        } else {
            ((*g).frealloc.expect("frealloc is null"))((*g).ud, block, osize, 0);
        }
    } else {
        result = ((*g).frealloc.expect("frealloc is null"))((*g).ud, block, osize, nsize);
        if result.is_null() && nsize > 0 {
            lua_d_throw(l, lua_Status::LUA_ERRMEM as i32);
        }
    }

    LUAU_ASSERT!((nsize == 0) == (result.is_null()));
    (*g).totalbytes = (*g).totalbytes.wrapping_sub(osize).wrapping_add(nsize);
    (*g).memcatbytes[memcat as usize] = (*g).memcatbytes[memcat as usize]
        .wrapping_add(nsize)
        .wrapping_sub(osize);

    if LUAU_UNLIKELY!((*g).cb.onallocate.is_some()) {
        ((*g).cb.onallocate.unwrap_unchecked())(l, osize, nsize);
    }

    result
}
