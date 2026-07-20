use crate::functions::freeobj::freeobj;
use crate::records::gc_object::GCObject;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_void;

#[allow(non_snake_case)]
pub(crate) unsafe fn deletegco(
    context: *mut c_void,
    page: *mut lua_Page,
    gco: *mut GCObject,
) -> bool {
    let L = context as *mut lua_State;
    freeobj(L, gco, page);
    true
}
