use crate::functions::freeclasspage::freeclasspage;
use crate::macros::asan_poison_memory_region::ASAN_POISON_MEMORY_REGION;
use crate::macros::debugpageset::debugpageset;
use crate::macros::metadata::metadata;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::global_state::global_State;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int, c_void};
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn freeblock(l: *mut lua_State, size_class: c_int, block: *mut c_void) {
    let g: *mut global_State = (*l).global;

    LUAU_ASSERT!(!block.is_null());
    let block = (block as *mut c_char).sub(crate::functions::newclasspage::k_block_header as usize)
        as *mut c_void;

    let page: *mut lua_Page = metadata!(block) as *mut lua_Page;
    LUAU_ASSERT!(!page.is_null() && (*page).busyBlocks > 0);
    LUAU_ASSERT!(
        ((*page).blockSize as usize)
            == (crate::records::size_class_config::kSizeClassConfig.sizeOfClass[size_class as usize]
                as usize
                + crate::functions::newclasspage::k_block_header as usize)
    );
    LUAU_ASSERT!(
        block >= (*page).data.as_ptr() as *mut c_void
            && (block as *mut c_char) < (page as *mut c_char).add((*page).pageSize as usize)
    );

    if (*page).freeList.is_null() && (*page).freeNext < 0 {
        LUAU_ASSERT!((*page).prev.is_null());
        LUAU_ASSERT!((*page).next.is_null());

        (*page).next = (*g).freepages[size_class as usize];
        if !(*page).next.is_null() {
            (*(*page).next).prev = page;
        }
        (*g).freepages[size_class as usize] = page;
    }

    metadata!(block) = (*page).freeList;
    (*page).freeList = block;

    ASAN_POISON_MEMORY_REGION!(block, (*page).blockSize as usize);

    (*page).busyBlocks -= 1;

    if (*page).busyBlocks == 0 {
        freeclasspage(
            l,
            (*g).freepages.as_mut_ptr(),
            debugpageset!(&mut (*g).allpages),
            page,
            size_class as u8,
        );
    }
}
