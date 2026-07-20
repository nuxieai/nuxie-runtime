use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_throw_ldo::lua_d_throw;
use crate::macros::asan_poison_memory_region::ASAN_POISON_MEMORY_REGION;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::global_state::global_State;
use crate::type_aliases::lua_state::lua_State;
use core::mem::offset_of;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn newpage(
    L: *mut lua_State,
    pageset: *mut *mut lua_Page,
    pageSize: core::ffi::c_int,
    blockSize: core::ffi::c_int,
    blockCount: core::ffi::c_int,
) -> *mut lua_Page {
    let g: *mut global_State = (*L).global;

    LUAU_ASSERT!(
        pageSize - (offset_of!(lua_Page, data) as core::ffi::c_int) >= blockSize * blockCount
    );

    let frealloc_fn = (*g).frealloc;
    let page = if let Some(f) = frealloc_fn {
        f((*g).ud, core::ptr::null_mut(), 0, pageSize as usize) as *mut lua_Page
    } else {
        core::ptr::null_mut()
    };

    if page.is_null() {
        lua_d_throw(L, crate::enums::lua_status::lua_Status::LUA_ERRMEM as i32);
    }

    ASAN_POISON_MEMORY_REGION!((*page).data.as_ptr(), (blockSize * blockCount) as usize);

    // setup page header
    (*page).prev = core::ptr::null_mut();
    (*page).next = core::ptr::null_mut();

    (*page).listprev = core::ptr::null_mut();
    (*page).listnext = core::ptr::null_mut();

    (*page).pageSize = pageSize;
    (*page).blockSize = blockSize;

    // note: we start with the last block in the page and move downward
    // either order would work, but that way we don't need to store the block count in the page
    // additionally, GC stores objects in singly linked lists, and this way the GC lists end up in increasing pointer order
    (*page).freeList = core::ptr::null_mut();
    (*page).freeNext = (blockCount - 1) * blockSize;
    (*page).busyBlocks = 0;

    if !pageset.is_null() {
        (*page).listnext = *pageset;
        if !(*page).listnext.is_null() {
            (*(*page).listnext).listprev = page;
        }
        *pageset = page;
    }

    page
}
