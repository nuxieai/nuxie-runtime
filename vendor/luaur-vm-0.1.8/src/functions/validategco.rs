use crate::functions::validateobj::validateobj;
use crate::records::gc_object::GCObject;
use crate::records::lua_page::lua_Page;
use crate::records::lua_state::lua_State;
use crate::type_aliases::global_state::global_State;
use core::ffi::c_void;

#[allow(non_snake_case)]
pub(crate) unsafe fn validategco(
    context: *mut c_void,
    _page: *mut lua_Page,
    gco: *mut GCObject,
) -> bool {
    let L = context as *mut lua_State;
    let g: *mut global_State = (*L).global;

    validateobj(g, gco);

    false
}
