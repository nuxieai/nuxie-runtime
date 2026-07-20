use crate::functions::newclasspage::newclasspage;
use crate::macros::asan_unpoison_memory_region::ASAN_UNPOISON_MEMORY_REGION;
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
pub(crate) unsafe fn newgcoblock(l: *mut lua_State, size_class: c_int) -> *mut c_void {
    let g: *mut global_State = (*l).global;
    let freegcopages = core::ptr::addr_of_mut!((*g).freegcopages) as *mut *mut lua_Page;
    let mut page: *mut lua_Page = *freegcopages.add(size_class as usize);

    // slow path: no page in the freelist, allocate a new one
    if page.is_null() {
        page = newclasspage(
            l,
            freegcopages,
            core::ptr::addr_of_mut!((*g).allgcopages),
            size_class as u8,
            false,
        );
    }

    LUAU_ASSERT!((*page).prev.is_null());
    LUAU_ASSERT!(!(*page).freeList.is_null() || (*page).freeNext >= 0);
    LUAU_ASSERT!(
        (*page).blockSize
            == crate::records::size_class_config::kSizeClassConfig.sizeOfClass[size_class as usize]
    );

    let block: *mut c_void;

    if (*page).freeNext >= 0 {
        let data = core::ptr::addr_of_mut!((*page).data) as *mut c_char;
        block = data.add((*page).freeNext as usize) as *mut c_void;
        ASAN_UNPOISON_MEMORY_REGION!(block, (*page).blockSize as usize);

        (*page).freeNext -= (*page).blockSize;
        (*page).busyBlocks += 1;
    } else {
        block = (*page).freeList;
        ASAN_UNPOISON_MEMORY_REGION!(
            (block as *mut c_char).add(core::mem::size_of::<GCheader>()) as *mut c_void,
            ((*page).blockSize as usize).wrapping_sub(core::mem::size_of::<GCheader>())
        );

        // when separate block metadata is not used, free list link is stored inside the block data itself
        (*page).freeList = *((block as *mut c_char).add(K_GCO_LINK_OFFSET) as *mut *mut c_void);
        (*page).busyBlocks += 1;
    }

    // if we allocate the last block out of a page, we need to remove it from free list
    if (*page).freeList.is_null() && (*page).freeNext < 0 {
        *freegcopages.add(size_class as usize) = (*page).next;
        if !(*page).next.is_null() {
            (*(*page).next).prev = core::ptr::null_mut();
        }
        (*page).next = core::ptr::null_mut();
    }

    block
}
