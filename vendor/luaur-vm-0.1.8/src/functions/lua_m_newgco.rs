use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_throw_ldo::luaD_throw;
use crate::functions::newgcoblock::newgcoblock;
use crate::functions::newpage::newpage;
use crate::macros::asan_unpoison_memory_region::ASAN_UNPOISON_MEMORY_REGION;
use crate::records::g_cheader::GCheader;
use crate::records::gc_object::GCObject;
use crate::records::lua_page::lua_Page;
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
pub unsafe fn luaM_newgco_(l: *mut lua_State, nsize: usize, memcat: u8) -> *mut GCObject {
    LUAU_ASSERT!(nsize >= K_GCO_LINK_OFFSET + core::mem::size_of::<*mut c_void>());

    let g = (*l).global;
    let nclass = sizeclass(nsize);

    let block = if nclass >= 0 {
        newgcoblock(l, nclass)
    } else {
        let page = newpage(
            l,
            core::ptr::addr_of_mut!((*g).allgcopages),
            (core::mem::offset_of!(lua_Page, data) + nsize) as i32,
            nsize as i32,
            1,
        );

        let block = (*page).data.as_mut_ptr() as *mut c_void;
        ASAN_UNPOISON_MEMORY_REGION!(block, (*page).blockSize as usize);

        (*page).freeNext -= (*page).blockSize;
        (*page).busyBlocks += 1;
        block
    };

    if block.is_null() && nsize > 0 {
        luaD_throw(l, lua_Status::LUA_ERRMEM as i32);
    }

    (*g).totalbytes = (*g).totalbytes.wrapping_add(nsize);
    (*g).memcatbytes[memcat as usize] = (*g).memcatbytes[memcat as usize].wrapping_add(nsize);

    if let Some(onallocate) = (*g).cb.onallocate {
        onallocate(l, 0, nsize);
    }

    block as *mut GCObject
}

#[allow(unused_imports)]
pub use luaM_newgco_ as lua_m_newgco_;
