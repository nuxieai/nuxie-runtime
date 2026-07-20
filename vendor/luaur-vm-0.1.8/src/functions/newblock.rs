use crate::functions::newclasspage::newclasspage;
use crate::macros::asan_unpoison_memory_region::ASAN_UNPOISON_MEMORY_REGION;
use crate::macros::debugpageset::debugpageset;
use crate::macros::metadata::metadata;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::global_state::global_State;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_void};
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn newblock(l: *mut lua_State, size_class: i32) -> *mut c_void {
    let g: *mut global_State = (*l).global;
    let mut page: *mut lua_Page = (*g).freepages[size_class as usize];

    // slow path: no page in the freelist, allocate a new one
    if page.is_null() {
        page = newclasspage(
            l,
            (*g).freepages.as_mut_ptr(),
            debugpageset!(&mut (*g).allpages),
            size_class as u8,
            true,
        );
    }

    LUAU_ASSERT!((*page).prev.is_null());
    LUAU_ASSERT!(!(*page).freeList.is_null() || (*page).freeNext >= 0);
    LUAU_ASSERT!(
        ((*page).blockSize as usize)
            == (crate::records::size_class_config::kSizeClassConfig.sizeOfClass[size_class as usize]
                as usize
                + crate::functions::newclasspage::k_block_header as usize)
    );

    let block: *mut c_void;

    if (*page).freeNext >= 0 {
        block = ((*page).data.as_mut_ptr() as *mut c_char).add((*page).freeNext as usize)
            as *mut c_void;
        ASAN_UNPOISON_MEMORY_REGION!(block, (*page).blockSize as usize);

        (*page).freeNext -= (*page).blockSize;
        (*page).busyBlocks += 1;
    } else {
        block = (*page).freeList;
        ASAN_UNPOISON_MEMORY_REGION!(block, (*page).blockSize as usize);

        (*page).freeList = metadata!(block);
        (*page).busyBlocks += 1;
    }

    // the first word in a block point back to the page
    metadata!(block) = page as *mut c_void;

    // if we allocate the last block out of a page, we need to remove it from free list
    if (*page).freeList.is_null() && (*page).freeNext < 0 {
        (*g).freepages[size_class as usize] = (*page).next;
        if !(*page).next.is_null() {
            (*(*page).next).prev = core::ptr::null_mut();
        }
        (*page).next = core::ptr::null_mut();
    }

    // the user data is right after the metadata
    (block as *mut c_char).add(crate::functions::newclasspage::k_block_header as usize)
        as *mut c_void
}
