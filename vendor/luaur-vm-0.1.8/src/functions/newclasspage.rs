use crate::functions::newpage::newpage;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_noinline::LUAU_NOINLINE;

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub(crate) const k_large_page_threshold: c_int = 1024;
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub(crate) const k_large_page_size: c_int = 65536;
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub(crate) const k_small_page_size: c_int = 16384;
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub(crate) const k_block_header: c_int = 8;

LUAU_NOINLINE! {
    #[allow(non_snake_case)]
    pub(crate) unsafe fn newclasspage(
        l: *mut lua_State,
        freepageset: *mut *mut lua_Page,
        pageset: *mut *mut lua_Page,
        size_class: u8,
        store_metadata: bool,
    ) -> *mut lua_Page {
        let size_of_class = crate::records::size_class_config::kSizeClassConfig.sizeOfClass[size_class as usize];
        let page_size = if size_of_class > k_large_page_threshold {
            k_large_page_size
        } else {
            k_small_page_size
        };
        let block_size = size_of_class + if store_metadata { k_block_header } else { 0 };
        let block_count = (page_size - core::mem::offset_of!(lua_Page, data) as c_int) / block_size;

        let page = newpage(l, pageset, page_size, block_size, block_count);

        LUAU_ASSERT!((*freepageset.add(size_class as usize)).is_null());
        *freepageset.add(size_class as usize) = page;

        page
    }
}
