//! Node: `cxx:Function:Luau.VM:VM/src/lmem.cpp:322:freeclasspage`
//! Source: `VM/src/lmem.cpp:322-331` (hand-ported)

use crate::functions::freepage::freepage;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::lua_state::lua_State;

pub(crate) unsafe fn freeclasspage(
    l: *mut lua_State,
    freepageset: *mut *mut lua_Page,
    pageset: *mut *mut lua_Page,
    page: *mut lua_Page,
    size_class: u8,
) {
    // remove page from freelist
    if !(*page).next.is_null() {
        (*(*page).next).prev = (*page).prev;
    }

    if !(*page).prev.is_null() {
        (*(*page).prev).next = (*page).next;
    } else {
        let idx = size_class as usize;
        if *freepageset.add(idx) == page {
            *freepageset.add(idx) = (*page).next;
        }
    }

    freepage(l, pageset, page);
}
