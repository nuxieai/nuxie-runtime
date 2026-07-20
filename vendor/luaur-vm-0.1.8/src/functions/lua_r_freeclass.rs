use crate::functions::lua_m_freegco::luaM_freegco_;
use crate::macros::lua_m_freearray::luaM_freearray;
use crate::records::gc_object::GCObject;
use crate::records::lua_page::lua_Page;
use crate::records::luau_class::LuauClass;
use crate::records::t_string::TString;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_r_freeclass(L: *mut lua_State, classobject: *mut LuauClass, page: *mut lua_Page) {
    let numberof_all_members = (*classobject).numberofallmembers;
    let numberof_instance_members = (*classobject).numberofinstancemembers;
    let static_member_count = numberof_all_members - numberof_instance_members;

    luaM_freearray!(
        L,
        (*classobject).staticmembers,
        static_member_count,
        TValue,
        (*classobject).memcat
    );
    luaM_freearray!(
        L,
        (*classobject).offsettomember,
        numberof_all_members,
        *mut TString,
        (*classobject).memcat
    );
    luaM_freegco_(
        L,
        classobject as *mut GCObject,
        core::mem::size_of::<LuauClass>(),
        (*classobject).memcat,
        page,
    );
}
