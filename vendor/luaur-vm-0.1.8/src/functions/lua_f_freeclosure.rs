use crate::functions::lua_m_freegco::luaM_freegco_;
use crate::macros::size_cclosure::size_cclosure;
use crate::macros::size_lclosure::size_lclosure;
use crate::records::closure::Closure;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_f_freeclosure(l: *mut lua_State, c: *mut Closure, page: *mut lua_Page) {
    unsafe {
        let size = if (*c).isC != 0 {
            size_cclosure((*c).nupvalues as core::ffi::c_int)
        } else {
            size_lclosure((*c).nupvalues as usize)
        };

        luaM_freegco_(
            l,
            c as *mut crate::records::gc_object::GcObject,
            size,
            (*c).hdr.memcat,
            page,
        );
    }
}
