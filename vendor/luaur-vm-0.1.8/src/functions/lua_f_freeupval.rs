use crate::functions::lua_m_freegco::luaM_freegco_;
use crate::macros::obj_2_gco::obj2gco;
use crate::records::lua_page::lua_Page;
use crate::records::up_val::UpVal;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_f_freeupval(L: *mut lua_State, uv: *mut UpVal, page: *mut lua_Page) {
    luaM_freegco_(
        L,
        uv as *mut crate::records::gc_object::GcObject,
        core::mem::size_of::<UpVal>(),
        (*uv).hdr.memcat,
        page,
    );
}
