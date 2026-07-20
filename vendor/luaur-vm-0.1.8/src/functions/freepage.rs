use crate::records::lua_page::lua_Page;
use crate::records::lua_state::lua_State;

pub(crate) unsafe fn freepage(L: *mut lua_State, pageset: *mut *mut lua_Page, page: *mut lua_Page) {
    let g = (*L).global;

    if !pageset.is_null() {
        // remove page from alllist
        if !(*page).listnext.is_null() {
            (*(*page).listnext).listprev = (*page).listprev;
        }

        if !(*page).listprev.is_null() {
            (*(*page).listprev).listnext = (*page).listnext;
        } else if *pageset == page {
            *pageset = (*page).listnext;
        }
    }

    // so long
    if let Some(frealloc) = (*g).frealloc {
        frealloc(
            (*g).ud,
            page as *mut core::ffi::c_void,
            (*page).pageSize as usize,
            0,
        );
    }
}
