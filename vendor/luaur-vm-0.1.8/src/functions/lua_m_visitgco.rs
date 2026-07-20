use crate::functions::lua_m_visitpage::lua_m_visitpage;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::global_state::global_State;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_void;

pub unsafe fn lua_m_visitgco(
    l: *mut lua_State,
    context: *mut c_void,
    visitor: *mut c_void, // function pointer: bool (*)(void* context, lua_Page* page, GCObject* gco)
) {
    let g: *mut global_State = (*l).global;

    let mut curr: *mut lua_Page = (*g).allgcopages;

    while !curr.is_null() {
        let next: *mut lua_Page = (*curr).listnext; // block visit might destroy the page

        lua_m_visitpage(curr, context, visitor);

        curr = next;
    }
}
