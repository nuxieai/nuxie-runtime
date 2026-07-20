use crate::functions::freeclasspage::freeclasspage;
use crate::macros::asan_poison_memory_region::ASAN_POISON_MEMORY_REGION;
use crate::records::g_cheader::GCheader;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::global_state::global_State;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int, c_void};
use luaur_common::macros::luau_assert::LUAU_ASSERT;

const K_GCO_LINK_OFFSET: usize =
    (core::mem::size_of::<GCheader>() + core::mem::size_of::<*mut c_void>() - 1)
        & !(core::mem::size_of::<*mut c_void>() - 1);

#[allow(non_snake_case)]
pub(crate) unsafe fn freegcoblock(
    l: *mut lua_State,
    size_class: c_int,
    block: *mut c_void,
    page: *mut lua_Page,
) {
    LUAU_ASSERT!(!page.is_null() && (*page).busyBlocks > 0);
    LUAU_ASSERT!(
        (*page).blockSize
            == crate::records::size_class_config::kSizeClassConfig.sizeOfClass[size_class as usize]
    );
    LUAU_ASSERT!(
        block >= core::ptr::addr_of!((*page).data) as *mut c_void
            && (block as *mut c_char) < (page as *mut c_char).add((*page).pageSize as usize)
    );

    let g: *mut global_State = (*l).global;
    let freegcopages = core::ptr::addr_of_mut!((*g).freegcopages) as *mut *mut lua_Page;

    // if the page wasn't in the page free list, it should be now since it got a block!
    if (*page).freeList.is_null() && (*page).freeNext < 0 {
        LUAU_ASSERT!((*page).prev.is_null());
        LUAU_ASSERT!((*page).next.is_null());

        (*page).next = *freegcopages.add(size_class as usize);
        if !(*page).next.is_null() {
            (*(*page).next).prev = page;
        }
        *freegcopages.add(size_class as usize) = page;
    }

    // when separate block metadata is not used, free list link is stored inside the block data itself
    *((block as *mut c_char).add(K_GCO_LINK_OFFSET) as *mut *mut c_void) = (*page).freeList;
    (*page).freeList = block;

    ASAN_POISON_MEMORY_REGION!(
        (block as *mut c_char).add(core::mem::size_of::<GCheader>()) as *mut c_void,
        ((*page).blockSize as usize).wrapping_sub(core::mem::size_of::<GCheader>())
    );

    (*page).busyBlocks -= 1;

    // if it's the last block in the page, we don't need the page
    if (*page).busyBlocks == 0 {
        freeclasspage(
            l,
            freegcopages,
            core::ptr::addr_of_mut!((*g).allgcopages),
            page,
            size_class as u8,
        );
    }
}
