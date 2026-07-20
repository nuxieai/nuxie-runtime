use crate::records::lua_page::lua_Page;

#[allow(non_snake_case)]
pub unsafe fn lua_m_getpageinfo(
    page: *mut lua_Page,
    page_blocks: *mut core::ffi::c_int,
    busy_blocks: *mut core::ffi::c_int,
    block_size: *mut core::ffi::c_int,
    page_size: *mut core::ffi::c_int,
) {
    let page_size_val = (*page).pageSize;
    let block_size_val = (*page).blockSize;

    *page_blocks = (page_size_val - core::mem::offset_of!(lua_Page, data) as core::ffi::c_int)
        / block_size_val;
    *busy_blocks = (*page).busyBlocks;
    *block_size = block_size_val;
    *page_size = page_size_val;
}
