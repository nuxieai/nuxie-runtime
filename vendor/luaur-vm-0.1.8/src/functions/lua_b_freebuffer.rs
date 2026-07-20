use crate::functions::lua_m_freegco::luaM_freegco_;
use crate::macros::obj_2_gco::obj2gco;
use crate::macros::sizebuffer::sizebuffer;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::buffer::Buffer;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_b_freebuffer(L: *mut lua_State, b: *mut Buffer, page: *mut lua_Page) {
    luaM_freegco_(
        L,
        obj2gco!(b) as *mut crate::records::gc_object::GcObject,
        sizebuffer((*b).len as usize),
        (*b).memcat,
        page,
    );
}
