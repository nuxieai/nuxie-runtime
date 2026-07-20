use crate::records::lua_page::lua_Page;
use core::ffi::{c_char, c_int};
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn lua_m_getpagewalkinfo(
    page: *mut lua_Page,
    start: *mut *mut c_char,
    end: *mut *mut c_char,
    busy_blocks: *mut c_int,
    block_size: *mut c_int,
) {
    let page_ref = &*page;
    let block_count =
        (page_ref.pageSize - core::mem::offset_of!(lua_Page, data) as c_int) / page_ref.blockSize;

    LUAU_ASSERT!(
        page_ref.freeNext >= -page_ref.blockSize
            && page_ref.freeNext <= (block_count - 1) * page_ref.blockSize
    );

    let data_ptr = page_ref.data.as_ptr() as *mut c_char;

    *start = data_ptr.add((page_ref.freeNext + page_ref.blockSize) as usize);
    *end = data_ptr.add((block_count * page_ref.blockSize) as usize);
    *busy_blocks = page_ref.busyBlocks;
    *block_size = page_ref.blockSize;
}
