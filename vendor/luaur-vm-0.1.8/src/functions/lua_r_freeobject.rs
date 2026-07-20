use crate::functions::lua_m_freegco::luaM_freegco_;
use crate::macros::lua_m_freearray::luaM_freearray;
use crate::records::gc_object::GCObject;
use crate::records::lua_page::lua_Page;
use crate::records::luau_object::LuauObject;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_r_freeobject(
    L: *mut lua_State,
    classinstance: *mut LuauObject,
    page: *mut lua_Page,
) {
    luaM_freearray!(
        L,
        (*classinstance).members,
        (*classinstance).numberofmembers,
        TValue,
        (*classinstance).memcat
    );

    luaM_freegco_(
        L,
        classinstance as *mut GCObject,
        core::mem::size_of::<LuauObject>(),
        (*classinstance).memcat,
        page,
    );
}
